import {Component, OnInit} from '@angular/core';
import {HttpClient} from '@angular/common/http';
import {MessageService} from 'primeng/api';
import {interval} from 'rxjs';
import {heartBeatAnimation} from 'angular-animations';

/* A search result row. */
export class Result {
  public trec_id: string;
  public corpus_id: string;
  public mixed_score: number;
  public record_score: number;
  public corpus_score: number;
  public url: string;
}

/* A search result Response */
export class SearchResult {
  public result: [Result];
  public result_count: number;
  public duration: number;
}

/* A stats request result */
export class StatsResult {
  public corpus_count: number;
  public record_count: number;
  public word_count: number;
}

/**
 * Main app component.
 */
@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.css'],
  animations: [
    heartBeatAnimation()
  ]
})
export class AppComponent implements OnInit {

  /**
   * The displayed result.
   */
  public results: [Result];

  /**
   * The title of the application.
   */
  public title: string = 'WARC Search';

  /**
   * The query.
   */
  public query: string = '';

  /**
   * Loading indicator.
   */
  public isLoading: boolean = false;

  /**
   * The search result count.
   */
  public result_count: number;

  /**
   * The duration in millis.
   */
  public duration: number;

  /**
   * The general status.
   */
  public stats: StatsResult;

  /**
   * Heartbeat.
   */
  public isAlive: boolean;

  /**
   * Trigger heartbeat animation.
   */
  public animateHeartbeat: boolean;

  /**
   * The server host.
   */
  private host: string = 'http://localhost:8000';

  constructor(private http: HttpClient, private readonly messageService: MessageService) {
  }

  public ngOnInit(): void {
    this.heartbeatDo();

    // Fetch the server stats.
    this.http.get(this.host + '/stats')
      .subscribe((data: StatsResult) => this.stats = data,
        (error => // Handle the error response.
          this.messageService.add({
            severity: 'error',
            summary: 'Failed to get database stats.',
            detail: 'Received code: ' + error.status
          })));

    // Heartbeat
    interval(5000)
      .subscribe((x) => {
        this.heartbeatDo();
      });
  }

  /**
   * Heartbeat trigger.
   */
  public heartbeatDo(): void {
    this.http.get(this.host + '/status')
      .subscribe((res: any) => {
        console.log('Pinged server, got 200 OK');
        this.isAlive = true;
        this.animate();
      }, (error) => {
        this.isAlive = false;
        this.messageService.add({
          severity: 'error',
          summary: 'Heartbeat failed...',
          detail: 'Received code: ' + error.status
        });
      });
  }

  /**
   * Animation trigger.
   */
  public animate(): void {
    this.animateHeartbeat = false;
    setTimeout(() => {
      this.animateHeartbeat = true;
    }, 2);
  }

  /**
   * Perform a search using the API.
   */
  public search(): void {
    // Indicate that it's loading.
    this.isLoading = true;
    this.http.post(this.host + '/query', {query: this.query})
      .subscribe((data: SearchResult) => {
        // Map the response.
        this.results = data.result;
        this.result_count = data.result_count;
        this.duration = data.duration;
        this.isLoading = false;
      }, (error) => {
        console.log(error);
        // Handle the error response.
        this.messageService.add({
          severity: 'error',
          summary: 'Oops, something went wrong with your request...',
          detail: 'Received code: ' + error.status
        });
        this.isLoading = false;
      });
  }
}
