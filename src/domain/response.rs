use crate::services::QueryWord;
use serde::Serialize;
use std::collections::HashMap;

/// A simple response container.
///
/// Contains the stats of the returned results.
#[derive(Debug, Serialize)]
pub struct ResponseContainer {
    /// The result container.
    pub result: Vec<SearchResult>,
    /// The total size of the response.
    pub result_count: usize,
    /// The search duration.
    pub duration: u128,
}

/// Simple Search Result structure.
///
/// Contains the individual result.
#[derive(Debug, Serialize, PartialEq, PartialOrd)]
pub struct SearchResult {
    /// The corpus ID from `corpus_index` table.
    pub corpus: String,
    /// The trec_id of the related document.
    pub trec_id: String,
    /// The linked URL.
    pub url: String,
    /// The corpus score as a total.
    pub corpus_score: f64,
    /// The record score.
    pub record_score: f64,
    /// The combined result of corpus and record.
    pub score_mixed: f64,
}

/// A record ranking temp helper.
pub struct RecordRanking {
    /// The related corpus name.
    corpus_name: String,
    /// The corpus id.
    corpus_id: i32,
    /// The related url.
    url: String,
    /// The trec id of the document.
    trec_id: String,
    /// The final rank.
    rank: f64,
    /// The numerator.
    x: f64,
    /// The denominator.
    y: f64,
}

/// A corpus ranking temp helper.
pub struct CorpusRanking {
    /// Name of the corpus (also used to identify it).
    corpus_name: String,
    /// Corpus id.
    corpus_id: i32,
    /// The final rank.
    rank: f64,
    /// The numerator.
    x: f64,
    /// The denominator.
    y: f64,
}

/// Build and calculate the rank of the returned results.
pub fn build_result(word_idx: Vec<QueryWord>) -> Vec<SearchResult> {
    // Temp holders.
    let mut record_ranking: Vec<RecordRanking> = vec![];
    let mut corpus_ranking: Vec<CorpusRanking> = vec![];

    // The denominator for each corpus.
    let mut denominator: HashMap<i32, f64> = HashMap::new();

    for word in word_idx.iter() {
        for corpus in word.tfidf.clone() {
            // Init to 0 (f64).
            denominator.insert(corpus.0, 0_f64);
        }
    }

    // Create the indexes for record and corpus.
    for word in word_idx.iter() {
        // Corpus.
        for corpus in word.word_corpus_tf_idf.iter() {
            denominator.insert(
                corpus.corpus_id,
                denominator[&corpus.corpus_id] + word.tfidf.get(&corpus.corpus_id).unwrap().powi(2),
            );
            if corpus_ranking
                .iter()
                .any(|x| x.corpus_name == corpus.corpus_name)
            {
                // The corpus ranking is already initiated. Update it.
                corpus_ranking
                    .iter_mut()
                    .find(|x| x.corpus_name == corpus.corpus_name)
                    .map(|x| {
                        x.x += word.tfidf.get(&corpus.corpus_id).unwrap() * corpus.tf_idf.unwrap();
                        x.y += corpus.tf_idf.unwrap().powi(2);
                    });
            } else {
                // Append the corpus.
                corpus_ranking.push(CorpusRanking {
                    corpus_name: String::from(&corpus.corpus_name),
                    corpus_id: corpus.corpus_id,
                    rank: 0_f64,
                    x: word.tfidf.get(&corpus.corpus_id).unwrap() * corpus.tf_idf.unwrap(),
                    y: corpus.tf_idf.unwrap().powi(2),
                });
            }
        }

        // Record.
        for record in word.word_record_tf_idf.iter() {
            if record_ranking.iter().any(|x| x.trec_id == record.trec_id) {
                // Update the record.
                record_ranking
                    .iter_mut()
                    .find(|x| x.trec_id == record.trec_id)
                    .map(|x| {
                        x.x += word.tfidf.get(&record.corpus_id).unwrap() * record.tf_idf.unwrap();
                        x.y += record.tf_idf.unwrap().powi(2);
                    });
            } else {
                // Append a new record.
                record_ranking.push(RecordRanking {
                    corpus_id: record.corpus_id,
                    url: String::from(&record.url),
                    corpus_name: String::from(&record.corpus_name),
                    trec_id: String::from(&record.trec_id),
                    rank: 0_f64,
                    x: word.tfidf.get(&record.corpus_id).unwrap() * record.tf_idf.unwrap(),
                    y: record.tf_idf.unwrap().powi(2),
                });
            }
        }
    }

    // Rank for Corpus
    for corpus in corpus_ranking.iter_mut() {
        corpus.rank = corpus.x / (denominator[&corpus.corpus_id].sqrt() * corpus.y.sqrt());
    }

    // Rank the record.
    for record in record_ranking.iter_mut() {
        record.rank = record.x / (denominator[&record.corpus_id].sqrt() * record.y.sqrt());
    }

    merge(record_ranking, corpus_ranking)
}

/// Result merger.
pub fn merge(
    record_ranking: Vec<RecordRanking>,
    corpus_ranking: Vec<CorpusRanking>,
) -> Vec<SearchResult> {
    let mut result: Vec<SearchResult> = vec![];

    // Rank each document separately.
    for record in record_ranking.iter() {
        // Get the rank of the related corpus (based on the ID).
        let corpus_rank = corpus_ranking
            .iter()
            .filter(|x| x.corpus_name == record.corpus_name)
            .map(|x| x.rank)
            .nth(0)
            .unwrap();
        // Create a new result and merge it.
        result.push(SearchResult {
            corpus: String::from(&record.corpus_name),
            trec_id: String::from(&record.trec_id),
            url: String::from(&record.url),
            record_score: record.rank,
            corpus_score: corpus_rank,
            score_mixed: (1_f64 - record.rank) * corpus_rank.log2(),
        });
    }

    // Sort descending.
    result.sort_by(|a, b| b.score_mixed.partial_cmp(&a.score_mixed).unwrap());

    result
}
