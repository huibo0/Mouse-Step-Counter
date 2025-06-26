use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Datelike};
use uuid::Uuid;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct MouseMovement {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub x: i32,
    pub y: i32,
    pub distance: f64,
    pub steps: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailyStats {
    pub id: String,
    pub date: String, // YYYY-MM-DD format
    pub total_steps: u32,
    pub total_distance: f64,
    pub movement_count: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonthlyStats {
    pub id: String,
    pub year_month: String, // YYYY-MM format
    pub total_steps: u32,
    pub total_distance: f64,
    pub total_days: u32,
    pub avg_steps_per_day: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeeklyStats {
    pub id: String,
    pub year_week: String, // YYYY-WW format (e.g., 2024-01 for first week)
    pub total_steps: u32,
    pub total_distance: f64,
    pub total_days: u32,
    pub avg_steps_per_day: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EventType {
    LongestSingleMove,      // 单次移动最远距离
    LongestWorkSession,     // 最长连续工作时长
    MostStepsInDay,        // 单日最多步数
    MostDistanceInDay,     // 单日最远距离
    WorkStreak,            // 连续工作天数
    SpeedDemon,            // 速度之王（短时间内大量移动）
    MarathonRunner,        // 马拉松跑者（长时间持续移动）
    PrecisionMaster,       // 精确大师（小幅度精确移动）
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AchievementEvent {
    pub id: String,
    pub event_type: EventType,
    pub value: f64,        // 成就值（距离、时长、步数等）
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: String,  // JSON格式的额外数据
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub event_type: EventType,
    pub threshold: f64,    // 触发阈值
    pub icon: String,      // 成就图标
    pub unlocked: bool,
    pub unlocked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkSession {
    pub id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_seconds: u64,
    pub total_steps: u32,
    pub total_distance: f64,
    pub movement_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunStats {
    pub longest_single_move: f64,
    pub longest_work_session_seconds: u64,
    pub most_steps_in_day: u32,
    pub most_distance_in_day: f64,
    pub current_work_streak: u32,
    pub total_achievements_unlocked: u32,
    pub fastest_movement_speed: f64, // 像素/秒
    pub total_work_sessions: u32,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let db = Database { conn };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<()> {
        // 鼠标运动记录表
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS mouse_movements (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                x INTEGER NOT NULL,
                y INTEGER NOT NULL,
                distance REAL NOT NULL,
                steps INTEGER NOT NULL
            )",
            [],
        )?;

        // 每日统计表
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS daily_stats (
                id TEXT PRIMARY KEY,
                date TEXT NOT NULL UNIQUE,
                total_steps INTEGER NOT NULL,
                total_distance REAL NOT NULL,
                movement_count INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // 每周统计表
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS weekly_stats (
                id TEXT PRIMARY KEY,
                year_week TEXT NOT NULL UNIQUE,
                total_steps INTEGER NOT NULL,
                total_distance REAL NOT NULL,
                total_days INTEGER NOT NULL,
                avg_steps_per_day REAL NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // 每月统计表
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS monthly_stats (
                id TEXT PRIMARY KEY,
                year_month TEXT NOT NULL UNIQUE,
                total_steps INTEGER NOT NULL,
                total_distance REAL NOT NULL,
                total_days INTEGER NOT NULL,
                avg_steps_per_day REAL NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // 成就事件表
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS achievement_events (
                id TEXT PRIMARY KEY,
                event_type TEXT NOT NULL,
                value REAL NOT NULL,
                description TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                metadata TEXT NOT NULL
            )",
            [],
        )?;

        // 成就表
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS achievements (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                event_type TEXT NOT NULL,
                threshold REAL NOT NULL,
                icon TEXT NOT NULL,
                unlocked INTEGER NOT NULL DEFAULT 0,
                unlocked_at TEXT,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        // 工作会话表
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS work_sessions (
                id TEXT PRIMARY KEY,
                start_time TEXT NOT NULL,
                end_time TEXT,
                duration_seconds INTEGER NOT NULL,
                total_steps INTEGER NOT NULL,
                total_distance REAL NOT NULL,
                movement_count INTEGER NOT NULL
            )",
            [],
        )?;

        // 初始化默认成就
        self.init_default_achievements()?;

        Ok(())
    }

    // 记录鼠标运动
    pub fn record_movement(&self, x: i32, y: i32, distance: f64, steps: u32) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();
        
        self.conn.execute(
            "INSERT INTO mouse_movements (id, timestamp, x, y, distance, steps) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                id,
                timestamp.to_rfc3339(),
                x,
                y,
                distance,
                steps
            ],
        )?;

        // --- 成就1：单次移动最远距离 ---
        let prev_max: f64 = self.conn.query_row(
            "SELECT COALESCE(MAX(distance), 0) FROM mouse_movements WHERE id != ?1",
            params![id],
            |row| row.get(0),
        )?;
        if distance > prev_max {
            let event = AchievementEvent {
                id: Uuid::new_v4().to_string(),
                event_type: EventType::LongestSingleMove,
                value: distance,
                description: format!("单次移动距离: {:.1}像素", distance),
                timestamp,
                metadata: serde_json::to_string(&serde_json::json!({
                    "x": x,
                    "y": y,
                    "distance": distance,
                })).unwrap(),
            };
            self.record_achievement_event(event)?;
        }

        // --- 原有每日统计等逻辑 ---
        self.update_daily_stats(&timestamp)?;
        
        Ok(())
    }

    // 更新每日统计
    fn update_daily_stats(&self, timestamp: &DateTime<Utc>) -> Result<()> {
        let date = timestamp.format("%Y-%m-%d").to_string();
        let now = Utc::now();

        // 获取今日的运动数据
        let (total_steps, total_distance, movement_count): (u32, f64, u32) = self.conn.query_row(
            "SELECT COALESCE(SUM(steps), 0), COALESCE(SUM(distance), 0), COUNT(*) 
             FROM mouse_movements 
             WHERE date(timestamp) = ?1",
            params![date],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                ))
            },
        )?;

        // 插入或更新每日统计
        self.conn.execute(
            "INSERT OR REPLACE INTO daily_stats 
             (id, date, total_steps, total_distance, movement_count, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                Uuid::new_v4().to_string(),
                date,
                total_steps,
                total_distance,
                movement_count,
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        )?;

        // --- 成就2：单日步数/距离 ---
        // 步数
        let prev_max_steps: u32 = self.conn.query_row(
            "SELECT COALESCE(MAX(total_steps), 0) FROM daily_stats WHERE date != ?1",
            params![date],
            |row| row.get(0),
        )?;
        if total_steps > prev_max_steps {
            let event = AchievementEvent {
                id: Uuid::new_v4().to_string(),
                event_type: EventType::MostStepsInDay,
                value: total_steps as f64,
                description: format!("单日步数: {}", total_steps),
                timestamp: Utc::now(),
                metadata: serde_json::to_string(&serde_json::json!({
                    "date": date,
                    "steps": total_steps,
                })).unwrap(),
            };
            self.record_achievement_event(event)?;
        }
        // 距离
        let prev_max_distance: f64 = self.conn.query_row(
            "SELECT COALESCE(MAX(total_distance), 0) FROM daily_stats WHERE date != ?1",
            params![date],
            |row| row.get(0),
        )?;
        if total_distance > prev_max_distance {
            let event = AchievementEvent {
                id: Uuid::new_v4().to_string(),
                event_type: EventType::MostDistanceInDay,
                value: total_distance,
                description: format!("单日距离: {:.1}像素", total_distance),
                timestamp: Utc::now(),
                metadata: serde_json::to_string(&serde_json::json!({
                    "date": date,
                    "distance": total_distance,
                })).unwrap(),
            };
            self.record_achievement_event(event)?;
        }

        // --- 成就4：连续工作天数 ---
        let current_streak: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM (
                SELECT date FROM daily_stats
                WHERE total_steps > 0
                ORDER BY date DESC
                LIMIT 30
            )",
            [],
            |row| row.get(0),
        )?;
        let prev_max_streak: u32 = self.conn.query_row(
            "SELECT COALESCE(MAX(value), 0) FROM achievement_events WHERE event_type = ?1",
            params![serde_json::to_string(&EventType::WorkStreak).unwrap()],
            |row| {
                let value: f64 = row.get(0)?;
                Ok(value as u32)
            },
        )?;
        if current_streak > prev_max_streak {
            let event = AchievementEvent {
                id: Uuid::new_v4().to_string(),
                event_type: EventType::WorkStreak,
                value: current_streak as f64,
                description: format!("连续工作天数: {}", current_streak),
                timestamp: Utc::now(),
                metadata: serde_json::to_string(&serde_json::json!({
                    "streak": current_streak,
                })).unwrap(),
            };
            self.record_achievement_event(event)?;
        }

        // --- 原有每周和每月统计 ---
        self.update_weekly_stats(&timestamp)?;
        self.update_monthly_stats(&timestamp)?;

        Ok(())
    }

    // 更新每周统计
    fn update_weekly_stats(&self, timestamp: &DateTime<Utc>) -> Result<()> {
        let year_week = format!("{}-{:02}", timestamp.year(), timestamp.iso_week().week());
        let now = Utc::now();

        // 获取本周的数据
        let week_start = timestamp.date_naive().week(chrono::Weekday::Mon).first_day();
        let week_end = week_start + chrono::Duration::days(6);

        let (total_steps, total_distance, total_days): (u32, f64, u32) = self.conn.query_row(
            "SELECT COALESCE(SUM(total_steps), 0), COALESCE(SUM(total_distance), 0), COUNT(*)
             FROM daily_stats 
             WHERE date BETWEEN ?1 AND ?2",
            params![week_start.format("%Y-%m-%d").to_string(), week_end.format("%Y-%m-%d").to_string()],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                ))
            },
        )?;

        let avg_steps_per_day = if total_days > 0 {
            total_steps as f64 / total_days as f64
        } else {
            0.0
        };

        self.conn.execute(
            "INSERT OR REPLACE INTO weekly_stats 
             (id, year_week, total_steps, total_distance, total_days, avg_steps_per_day, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                Uuid::new_v4().to_string(),
                year_week,
                total_steps,
                total_distance,
                total_days,
                avg_steps_per_day,
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    // 更新每月统计
    fn update_monthly_stats(&self, timestamp: &DateTime<Utc>) -> Result<()> {
        let year_month = timestamp.format("%Y-%m").to_string();
        let now = Utc::now();

        // 获取本月的数据
        let (total_steps, total_distance, total_days): (u32, f64, u32) = self.conn.query_row(
            "SELECT COALESCE(SUM(total_steps), 0), COALESCE(SUM(total_distance), 0), COUNT(*)
             FROM daily_stats 
             WHERE strftime('%Y-%m', date) = ?1",
            params![year_month],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                ))
            },
        )?;

        let avg_steps_per_day = if total_days > 0 {
            total_steps as f64 / total_days as f64
        } else {
            0.0
        };

        self.conn.execute(
            "INSERT OR REPLACE INTO monthly_stats 
             (id, year_month, total_steps, total_distance, total_days, avg_steps_per_day, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                Uuid::new_v4().to_string(),
                year_month,
                total_steps,
                total_distance,
                total_days,
                avg_steps_per_day,
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    // 获取每日统计数据
    pub fn get_daily_stats(&self, days: i32) -> Result<Vec<DailyStats>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, date, total_steps, total_distance, movement_count, created_at, updated_at
             FROM daily_stats 
             ORDER BY date DESC 
             LIMIT ?1"
        )?;

        let rows = stmt.query_map(params![days], |row| {
            Ok(DailyStats {
                id: row.get(0)?,
                date: row.get(1)?,
                total_steps: row.get(2)?,
                total_distance: row.get(3)?,
                movement_count: row.get(4)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                    .unwrap()
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .unwrap()
                    .with_timezone(&Utc),
            })
        })?;

        let mut stats = Vec::new();
        for row in rows {
            stats.push(row?);
        }
        Ok(stats)
    }

    // 获取每周统计数据
    pub fn get_weekly_stats(&self, weeks: i32) -> Result<Vec<WeeklyStats>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, year_week, total_steps, total_distance, total_days, avg_steps_per_day, created_at, updated_at
             FROM weekly_stats 
             ORDER BY year_week DESC 
             LIMIT ?1"
        )?;

        let rows = stmt.query_map(params![weeks], |row| {
            Ok(WeeklyStats {
                id: row.get(0)?,
                year_week: row.get(1)?,
                total_steps: row.get(2)?,
                total_distance: row.get(3)?,
                total_days: row.get(4)?,
                avg_steps_per_day: row.get(5)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .unwrap()
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .unwrap()
                    .with_timezone(&Utc),
            })
        })?;

        let mut stats = Vec::new();
        for row in rows {
            stats.push(row?);
        }
        Ok(stats)
    }

    // 获取每月统计数据
    pub fn get_monthly_stats(&self, months: i32) -> Result<Vec<MonthlyStats>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, year_month, total_steps, total_distance, total_days, avg_steps_per_day, created_at, updated_at
             FROM monthly_stats 
             ORDER BY year_month DESC 
             LIMIT ?1"
        )?;

        let rows = stmt.query_map(params![months], |row| {
            Ok(MonthlyStats {
                id: row.get(0)?,
                year_month: row.get(1)?,
                total_steps: row.get(2)?,
                total_distance: row.get(3)?,
                total_days: row.get(4)?,
                avg_steps_per_day: row.get(5)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .unwrap()
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .unwrap()
                    .with_timezone(&Utc),
            })
        })?;

        let mut stats = Vec::new();
        for row in rows {
            stats.push(row?);
        }
        Ok(stats)
    }

    // 获取今日统计
    pub fn get_today_stats(&self) -> Result<Option<DailyStats>> {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        
        let result = self.conn.query_row(
            "SELECT id, date, total_steps, total_distance, movement_count, created_at, updated_at
             FROM daily_stats 
             WHERE date = ?1",
            params![today],
            |row| {
                Ok(DailyStats {
                    id: row.get(0)?,
                    date: row.get(1)?,
                    total_steps: row.get(2)?,
                    total_distance: row.get(3)?,
                    movement_count: row.get(4)?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .unwrap()
                        .with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                        .unwrap()
                        .with_timezone(&Utc),
                })
            },
        );

        match result {
            Ok(stats) => Ok(Some(stats)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    // 获取总统计信息
    pub fn get_total_stats(&self) -> Result<(u32, f64, u32)> {
        let (total_steps, total_distance, total_days): (u32, f64, u32) = self.conn.query_row(
            "SELECT COALESCE(SUM(total_steps), 0), COALESCE(SUM(total_distance), 0), COUNT(*)
             FROM daily_stats",
            [],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                ))
            },
        )?;

        Ok((total_steps, total_distance, total_days))
    }

    // 清空所有统计数据
    pub fn clear_all_stats(&self) -> Result<()> {
        self.conn.execute("DELETE FROM mouse_movements", [])?;
        self.conn.execute("DELETE FROM daily_stats", [])?;
        self.conn.execute("DELETE FROM weekly_stats", [])?;
        self.conn.execute("DELETE FROM monthly_stats", [])?;
        self.conn.execute("DELETE FROM work_sessions", [])?;
        Ok(())
    }

    // 初始化默认成就
    fn init_default_achievements(&self) -> Result<()> {
        let now = Utc::now();
        let default_achievements = vec![
            ("longest_move_100", "短跑健将", "单次移动距离达到100像素", EventType::LongestSingleMove, 100.0, "🏃"),
            ("longest_move_500", "长跑健将", "单次移动距离达到500像素", EventType::LongestSingleMove, 500.0, "🏃‍♂️"),
            ("longest_move_1000", "马拉松选手", "单次移动距离达到1000像素", EventType::LongestSingleMove, 1000.0, "🏃‍♀️"),
            ("work_session_30min", "专注工作者", "连续工作30分钟", EventType::LongestWorkSession, 1800.0, "⏰"),
            ("work_session_1hour", "深度工作者", "连续工作1小时", EventType::LongestWorkSession, 3600.0, "💼"),
            ("work_session_2hour", "工作狂", "连续工作2小时", EventType::LongestWorkSession, 7200.0, "🔥"),
            ("daily_steps_1000", "千步达人", "单日步数达到1000步", EventType::MostStepsInDay, 1000.0, "👣"),
            ("daily_steps_5000", "万步达人", "单日步数达到5000步", EventType::MostStepsInDay, 5000.0, "👟"),
            ("daily_steps_10000", "运动健将", "单日步数达到10000步", EventType::MostStepsInDay, 10000.0, "🏆"),
            ("daily_distance_1000", "短途旅行", "单日移动距离达到1000像素", EventType::MostDistanceInDay, 1000.0, "🚶"),
            ("daily_distance_5000", "长途旅行", "单日移动距离达到5000像素", EventType::MostDistanceInDay, 5000.0, "🚶‍♂️"),
            ("work_streak_3", "连续工作3天", "连续工作3天", EventType::WorkStreak, 3.0, "📅"),
            ("work_streak_7", "连续工作一周", "连续工作7天", EventType::WorkStreak, 7.0, "📆"),
            ("work_streak_30", "工作狂人", "连续工作30天", EventType::WorkStreak, 30.0, "🎯"),
        ];

        for (id, name, description, event_type, threshold, icon) in default_achievements {
            // 检查成就是否已存在
            let exists: i32 = self.conn.query_row(
                "SELECT COUNT(*) FROM achievements WHERE id = ?1",
                params![id],
                |row| row.get(0),
            ).unwrap_or(0);

            if exists == 0 {
                self.conn.execute(
                    "INSERT INTO achievements (id, name, description, event_type, threshold, icon, unlocked, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7)",
                    params![
                        id,
                        name,
                        description,
                        serde_json::to_string(&event_type).unwrap(),
                        threshold,
                        icon,
                        now.to_rfc3339(),
                    ],
                )?;
            }
        }

        Ok(())
    }

    // 记录成就事件
    pub fn record_achievement_event(&self, event: AchievementEvent) -> Result<()> {
        self.conn.execute(
            "INSERT INTO achievement_events (id, event_type, value, description, timestamp, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                event.id,
                serde_json::to_string(&event.event_type).unwrap(),
                event.value,
                event.description,
                event.timestamp.to_rfc3339(),
                event.metadata,
            ],
        )?;

        // 检查是否解锁新成就
        self.check_achievements(&event.event_type, event.value)?;

        Ok(())
    }

    // 检查成就解锁
    fn check_achievements(&self, event_type: &EventType, value: f64) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, threshold FROM achievements 
             WHERE event_type = ?1 AND unlocked = 0 AND threshold <= ?2"
        )?;

        let rows = stmt.query_map(params![serde_json::to_string(event_type).unwrap(), value], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, f64>(2)?,
            ))
        })?;

        let now = Utc::now();
        for row in rows {
            let (id, name, threshold) = row?;
            
            // 解锁成就
            self.conn.execute(
                "UPDATE achievements SET unlocked = 1, unlocked_at = ?1 WHERE id = ?2",
                params![now.to_rfc3339(), id],
            )?;

            println!("🎉 解锁成就: {} (阈值: {})", name, threshold);
        }

        Ok(())
    }

    // 开始牛马计时器（原工作会话）
    pub fn start_work_session(&self) -> Result<String> {
        let now = Utc::now();
        // 关闭所有未结束的会话
        self.conn.execute(
            "UPDATE work_sessions SET end_time = ?1, duration_seconds = 0, total_steps = 0, total_distance = 0, movement_count = 0 WHERE end_time IS NULL",
            params![now.to_rfc3339()],
        )?;
        let id = Uuid::new_v4().to_string();
        self.conn.execute(
            "INSERT INTO work_sessions (id, start_time, duration_seconds, total_steps, total_distance, movement_count)
             VALUES (?1, ?2, 0, 0, 0, 0)",
            params![id, now.to_rfc3339()],
        )?;
        Ok(id)
    }

    // 结束工作会话
    pub fn end_work_session(&self, session_id: &str, total_steps: u32, total_distance: f64, movement_count: u32) -> Result<()> {
        let now = Utc::now();
        
        // 计算会话时长
        let (start_time,): (String,) = self.conn.query_row(
            "SELECT start_time FROM work_sessions WHERE id = ?1",
            params![session_id],
            |row| Ok((row.get(0)?,)),
        )?;

        let start = DateTime::parse_from_rfc3339(&start_time).unwrap().with_timezone(&Utc);
        let duration = now.signed_duration_since(start).num_seconds() as u64;

        self.conn.execute(
            "UPDATE work_sessions 
             SET end_time = ?1, duration_seconds = ?2, total_steps = ?3, total_distance = ?4, movement_count = ?5
             WHERE id = ?6",
            params![now.to_rfc3339(), duration, total_steps, total_distance, movement_count, session_id],
        )?;

        // --- 成就3：连续工作时长 ---
        let event = AchievementEvent {
            id: Uuid::new_v4().to_string(),
            event_type: EventType::LongestWorkSession,
            value: duration as f64,
            description: format!("连续工作{}分钟", duration / 60),
            timestamp: now,
            metadata: serde_json::to_string(&serde_json::json!({
                "session_id": session_id,
                "duration_minutes": duration / 60,
            })).unwrap(),
        };
        self.record_achievement_event(event)?;

        Ok(())
    }

    // 获取趣味统计
    pub fn get_fun_stats(&self) -> Result<FunStats> {
        // 最长单次移动
        let longest_single_move: f64 = self.conn.query_row(
            "SELECT COALESCE(MAX(distance), 0) FROM mouse_movements",
            [],
            |row| row.get(0),
        )?;

        // 最长工作会话
        let longest_work_session_seconds: u64 = self.conn.query_row(
            "SELECT COALESCE(MAX(duration_seconds), 0) FROM work_sessions WHERE end_time IS NOT NULL",
            [],
            |row| row.get(0),
        )?;

        // 单日最多步数
        let most_steps_in_day: u32 = self.conn.query_row(
            "SELECT COALESCE(MAX(total_steps), 0) FROM daily_stats",
            [],
            |row| row.get(0),
        )?;

        // 单日最远距离
        let most_distance_in_day: f64 = self.conn.query_row(
            "SELECT COALESCE(MAX(total_distance), 0) FROM daily_stats",
            [],
            |row| row.get(0),
        )?;

        // 已解锁成就数量
        let total_achievements_unlocked: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM achievements WHERE unlocked = 1",
            [],
            |row| row.get(0),
        )?;

        // 总工作会话数
        let total_work_sessions: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM work_sessions WHERE end_time IS NOT NULL",
            [],
            |row| row.get(0),
        )?;

        // 计算当前工作连续天数（修复版本）
        let current_work_streak: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM (
                SELECT DISTINCT date FROM daily_stats 
                WHERE total_steps > 0 
                ORDER BY date DESC 
                LIMIT 30
            )",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        // 最快移动速度（简化计算）
        let fastest_movement_speed: f64 = self.conn.query_row(
            "SELECT COALESCE(MAX(distance), 0) FROM mouse_movements",
            [],
            |row| row.get(0),
        )?;

        Ok(FunStats {
            longest_single_move,
            longest_work_session_seconds,
            most_steps_in_day,
            most_distance_in_day,
            current_work_streak,
            total_achievements_unlocked,
            fastest_movement_speed,
            total_work_sessions,
        })
    }

    // 获取所有成就
    pub fn get_achievements(&self) -> Result<Vec<Achievement>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, event_type, threshold, icon, unlocked, unlocked_at, created_at
             FROM achievements ORDER BY threshold ASC"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Achievement {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                event_type: serde_json::from_str(&row.get::<_, String>(3)?).unwrap(),
                threshold: row.get(4)?,
                icon: row.get(5)?,
                unlocked: row.get(6)?,
                unlocked_at: row.get::<_, Option<String>>(7)?
                    .map(|s| DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc)),
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .unwrap()
                    .with_timezone(&Utc),
            })
        })?;

        let mut achievements = Vec::new();
        for row in rows {
            achievements.push(row?);
        }
        Ok(achievements)
    }

    // 清理旧的鼠标移动数据（保留最近一周）
    pub fn cleanup_old_movements(&self) -> Result<()> {
        let one_week_ago = Utc::now() - chrono::Duration::days(7);
        
        let deleted_count = self.conn.execute(
            "DELETE FROM mouse_movements WHERE timestamp < ?1",
            params![one_week_ago.to_rfc3339()],
        )?;

        println!("🧹 清理了 {} 条旧的鼠标移动记录", deleted_count);
        Ok(())
    }

    // 获取牛马计时器（原工作会话）列表，默认只取最近3条
    pub fn get_work_sessions(&self, limit: usize) -> Result<Vec<WorkSession>> {
        let limit = if limit == 0 { 3 } else { limit };
        let mut stmt = self.conn.prepare(
            "SELECT id, start_time, end_time, duration_seconds, total_steps, total_distance, movement_count
             FROM work_sessions
             ORDER BY start_time DESC
             LIMIT ?1"
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            let start_time = DateTime::parse_from_rfc3339(&row.get::<_, String>(1)?).unwrap().with_timezone(&Utc);
            let end_time = row.get::<_, Option<String>>(2)?
                .map(|s| DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc));
            let duration_seconds = if end_time.is_none() {
                let now = Utc::now();
                now.signed_duration_since(start_time).num_seconds() as u64
            } else {
                row.get(3)?
            };
            Ok(WorkSession {
                id: row.get(0)?,
                start_time,
                end_time,
                duration_seconds,
                total_steps: row.get(4)?,
                total_distance: row.get(5)?,
                movement_count: row.get(6)?,
            })
        })?;
        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }

    // 更新牛马计时器进度（不结束会话）
    pub fn update_work_session_progress(&self, session_id: &str, total_steps: u32, total_distance: f64, movement_count: u32) -> Result<()> {
        self.conn.execute(
            "UPDATE work_sessions SET total_steps = ?1, total_distance = ?2, movement_count = ?3 WHERE id = ?4 AND end_time IS NULL",
            params![total_steps, total_distance, movement_count, session_id],
        )?;
        Ok(())
    }
} 