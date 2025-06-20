export interface WaterReminderConfig {
  daily_glasses: number;
  reminder_interval_hours: number;
  custom_reminder_times: string[];
  enabled: boolean;
}

export interface WaterReminderState {
  config: WaterReminderConfig;
  last_reminder_time: number;
  glasses_drunk_today: number;
  last_reset_date: string;
  current_period_water_drunk: boolean;
  last_water_period_start: number;
} 