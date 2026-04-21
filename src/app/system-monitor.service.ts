import { Injectable, NgZone, inject } from "@angular/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { Observable, shareReplay } from "rxjs";

export type SystemStats = {
  cpuUsage: number;
  memoryUsed: number;
  memoryTotal: number;
  networkDownloadBps: number;
};

@Injectable({ providedIn: "root" })
export class SystemMonitorService {
  private readonly ngZone = inject(NgZone);

  readonly stats$ = new Observable<SystemStats>((subscriber) => {
    let unlisten: UnlistenFn | undefined;
    let closed = false;

    void listen<SystemStats>("system-stats", (event) => {
      this.ngZone.run(() => {
        subscriber.next(event.payload);
      });
    })
      .then((cleanup) => {
        if (closed) {
          cleanup();
          return;
        }

        unlisten = cleanup;
      })
      .catch((error) => {
        this.ngZone.run(() => {
          subscriber.error(error);
        });
      });

    return () => {
      closed = true;
      void unlisten?.();
    };
  }).pipe(
    shareReplay({
      bufferSize: 1,
      refCount: true,
    }),
  );
}
