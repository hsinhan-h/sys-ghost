import { AsyncPipe, DecimalPipe, NgIf } from "@angular/common";
import { Component, inject } from "@angular/core";
import { RouterOutlet } from "@angular/router";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { map } from "rxjs";
import { SystemMonitorService } from "./system-monitor.service";

@Component({
  selector: "app-root",
  imports: [AsyncPipe, DecimalPipe, NgIf, RouterOutlet],
  templateUrl: "./app.component.html",
  styleUrl: "./app.component.css",
})
export class AppComponent {
  private readonly appWindow = getCurrentWindow();
  private readonly systemMonitorService = inject(SystemMonitorService);
  readonly systemStats$ = this.systemMonitorService.stats$;
  readonly dashboardStats$ = this.systemStats$.pipe(
    map((stats) => ({
      ...stats,
      memoryUsedGb: stats.memoryUsed / 1024 / 1024 / 1024,
      memoryTotalGb: stats.memoryTotal / 1024 / 1024 / 1024,
      memoryUsagePercent:
        stats.memoryTotal > 0 ? (stats.memoryUsed / stats.memoryTotal) * 100 : 0,
    })),
  );

  startDrag(event: MouseEvent): void {
    if (event.button !== 0) {
      return;
    }

    void this.appWindow.startDragging();
  }
}
