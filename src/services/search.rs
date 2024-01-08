use crate::domain::{build_result, SearchResult};
use convert_case::{Case, Casing};
use log::{error, warn};
use sqlx::PgPool;
use std::collections::HashMap;

/// The query record information.
pub struct QueryWordRecordInfo {
    // The record's id.
    pub record_id: i32,
    // The word.
    pub word: String,
    // The trec id of the record.
    pub trec_id: String,
    // The linked corpus name.
    pub corpus_name: String,
    // The corpus id.
    pub corpus_id: i32,
    // The tf_idf of the word.
    pub tf_idf: Option<f64>,
    // The linked URL.
    pub url: String,
    // The appearances of the word in the record.
    pub appearances_r: i32,
    // Total words in the record.
    pub total_words_r: Option<i32>,
}

/// The query word corpus information.
pub struct QueryWordCorpusInfo {
    // The corpus id.
    pub corpus_id: i32,
    // The corpus name.
    pub corpus_name: String,
    // The number of appearances of the word in the corpus.
    pub appearances_c: i32,
    // The total number of words in the corpus.
    pub total_words_c: Option<i64>,
    // The word.
    pub word: String,
    // The tf_idf (cpre-calculated).
    pub tf_idf: Option<f64>,
}

/// A storage place for a query word.
pub struct QueryWord {
    // The query word.
    pub word: String,
    // The tf-idf of the word for each corpus.
    pub tfidf: HashMap<i32, f64>,
    // The vector if TF-IDF compared to the corpus.
    pub word_corpus_tf_idf: Vec<QueryWordCorpusInfo>,
    // The vector of TF-IDF compared to the records.
    pub word_record_tf_idf: Vec<QueryWordRecordInfo>,
}

/// Implementation for `QueryWord` structure.
impl QueryWord {
    /// Create a new `QueryWord` instance.
    ///
    /// # Arguments
    ///
    /// * word      - The word to lik it to.
    pub fn new(word: String) -> QueryWord {
        QueryWord {
            word: String::from(word),
            tfidf: HashMap::new(),
            word_corpus_tf_idf: vec![],
            word_record_tf_idf: vec![],
        }
    }

    /// Create the query word TF-IDF for each corpus that contains it.
    ///
    /// # Arguments
    ///
    /// * pool      - The PostgreSQL connection pool.
    pub async fn get_corpus_tf_idf(&mut self, pool: &PgPool) -> Result<(), sqlx::Error> {
        let result: Vec<QueryWordCorpusInfo> = sqlx::query_as!(
            QueryWordCorpusInfo,
            r#"
            SELECT ci.id                            AS corpus_id,
                   ci.name                          AS corpus_name,
                   wci.word                         AS word,
                   wci.appearances                  AS appearances_c,
                   (SELECT SUM(wcii.appearances)
                    FROM word_corpus_index wcii
                    WHERE wcii.corpus = wci.corpus) AS total_words_c,
                   wci.tf * wci.idf                 AS tf_idf
            FROM word_corpus_index wci
                     JOIN corpus_info ci
                          ON wci.corpus = ci.id
            WHERE wci.word = $1
        "#,
            &self.word
        )
        .fetch_all(pool)
        .await?;

        self.word_corpus_tf_idf = result;

        Ok(())
    }

    /// Get the per-corpus TF-IDF for the word.
    ///
    /// # Arguments
    ///
    /// * pool      - The PostgreSQL connection pool.
    pub async fn get_record_tf_idf(&mut self, pool: &PgPool) -> Result<(), sqlx::Error> {
        let result: Vec<QueryWordRecordInfo> = sqlx::query_as!(
            QueryWordRecordInfo,
            r#"
            SELECT wri.record       AS record_id,
                   wri.word         AS word,
                   ri.trec_id       AS trec_id,
                   ci.name          AS corpus_name,
                   ci.id            AS corpus_id,
                   wri.tf * wri.idf AS tf_idf,
                   ri.uri           AS url,
                   wri.appearances  AS appearances_r,
                   ri.total_words   AS total_words_r
            FROM word_record_index wri
                     JOIN record_index ri
                          ON wri.record = ri.id
                     JOIN corpus_info ci
                          ON ri.corpus_id = ci.id
            WHERE wri.word = $1
        "#,
            &self.word
        )
        .fetch_all(pool)
        .await?;

        self.word_record_tf_idf = result;

        Ok(())
    }
}

/// Calculate the TF-IDF of the given query word against the total corpus size.
///
/// # Arguments
///
/// * query_word        - The query word.
/// * total_words       - The total words in the query.
/// * pool              - The PostgreSQL connection pool.
pub async fn calculate_tf_idf_for_query_word(
    query_word: &mut QueryWord,
    total_words: f64,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT id
        FROM corpus_info
        WHERE id IN (SELECT corpus FROM word_corpus_index WHERE word = $1)
        "#,
        &query_word.word
    )
    .fetch_all(pool)
    .await
    .map_err(|e| error!("{}", e));

    let ids = result.unwrap();

    let mut weights: HashMap<i32, f64> = HashMap::new();

    for id in ids.iter() {
        // Query
        let result = sqlx::query!(
            r#"
        SELECT (1 + log((SELECT COUNT(*) FROM record_index fi) / COUNT(*)))
                   AS idf
        FROM record_index ci
        WHERE ci.id IN (SELECT wci.record FROM word_record_index wci WHERE wci.word = $1)
        AND ci.corpus_id = $2
        "#,
            &query_word.word,
            &id.id
        )
        .fetch_one(pool)
        .await
        .map_err(|e| {
            error!("{}", e);
        });

        // Match the result
        match result {
            Ok(res) => match res.idf {
                Some(idf) => weights.insert(id.id, (1_f64 / total_words as f64) * idf),
                None => weights.insert(id.id, 0_f64),
            },
            Err(_) => weights.insert(id.id, 0_f64),
        };
    }

    query_word.tfidf = weights;

    // Just indicate that it finished successfully.
    Ok(())
}

/// Initial search layer.
///
/// Handles splitting the query into words, finding and excluding the stopwords etc.
///
/// # Arguments
///
/// * pool      - The PostgreSQL connection pool.
/// *query      - The query to execute.
pub async fn perform_search(pool: &PgPool, query: &str) -> Result<Vec<SearchResult>, sqlx::Error> {
    // Split the word into individual tokens.
    let split = query.split_whitespace();
    let wvector = split.collect::<Vec<&str>>();

    // Create an object for each word in the query.
    let mut query_words: Vec<QueryWord> = vec![];
    for word in wvector {
        if !is_stopword(word, pool).await? {
            query_words.push(QueryWord::new(word.to_case(Case::Lower)));
        } else {
            warn!("Detected stopword: {}", &word);
        }
    }

    let total_words_in_query: f64 = query_words.len() as f64;
    for word in query_words.iter_mut() {
        calculate_tf_idf_for_query_word(word, total_words_in_query, pool)
            .await
            .expect(&format!(
                "Failed to calculate TF-IDF for query word {}",
                word.word
            ));
        // Get the corpus TF-IDF
        word.get_corpus_tf_idf(pool)
            .await
            .expect("Failed to compute TF-IDF for word for each corpus.");
        word.get_record_tf_idf(pool)
            .await
            .expect("Failed to compute TF-IDF for word for each record.");
    }

    let result = build_result(query_words);

    Ok(result)
}

/// Check if the given word is a stopword (frequency over 0.9).
///
/// Extremely useful in case of queries that could return the entire database.
///
/// # Arguments
///
/// * word      - The word to search for.
/// * pool      - The PostgreSQL connection pool.
pub async fn is_stopword(word: &str, pool: &PgPool) -> Result<bool, sqlx::Error> {
    // First check if the given word exists.
    let result = sqlx::query!(
        r#"
    SELECT EXISTS(SELECT * FROM word_index wi WHERE wi.word = $1);
    "#,
        &word
    )
    .fetch_one(pool)
    .await?;

    // Match the result
    match result.exists {
        Some(s) => {
            if !s {
                return Ok(false);
            }
        }
        None => return Ok(false),
    }

    let result = sqlx::query!(
        r#"
    SELECT (CASE
                WHEN wi.frequency > 0.9
                    THEN TRUE
                ELSE FALSE
        END) AS is_stopword
    FROM word_index wi
    WHERE wi.word = $1
    "#,
        &word
    )
    .fetch_one(pool)
    .await?;

    // Match the result
    Ok(match result.is_stopword {
        Some(s) => s,
        None => false,
    })
}
