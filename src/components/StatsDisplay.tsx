import { useState, useEffect } from 'react';
import {
  Box,
  Typography,
  Card,
  CardContent,
  Tabs,
  Tab,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  Chip,
  LinearProgress,
  IconButton,
  Tooltip,
  Button,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Alert,
} from '@mui/material';
import {
  TrendingUp,
  CalendarToday,
  ShowChart,
  Refresh,
  Timeline,
  DeleteForever,
} from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/tauri';
import { emit } from '@tauri-apps/api/event';

interface DailyStats {
  id: string;
  date: string;
  total_steps: number;
  total_distance: number;
  movement_count: number;
  created_at: string;
  updated_at: string;
}

interface WeeklyStats {
  id: string;
  year_week: string;
  total_steps: number;
  total_distance: number;
  total_days: number;
  avg_steps_per_day: number;
  created_at: string;
  updated_at: string;
}

interface MonthlyStats {
  id: string;
  year_month: string;
  total_steps: number;
  total_distance: number;
  total_days: number;
  avg_steps_per_day: number;
  created_at: string;
  updated_at: string;
}

interface TotalStats {
  total_steps: number;
  total_distance: number;
  total_days: number;
}

interface WorkSession {
  id: string;
  start_time: string;
  end_time: string | null;
  duration_seconds: number;
  total_steps: number;
  total_distance: number;
  movement_count: number;
}

interface FunStats {
  longest_single_move: number;
  longest_work_session_seconds: number;
  most_steps_in_day: number;
  most_distance_in_day: number;
  current_work_streak: number;
  total_achievements_unlocked: number;
  fastest_movement_speed: number;
  total_work_sessions: number;
}

export function StatsDisplay() {
  const [activeTab, setActiveTab] = useState(0);
  const [dailyStats, setDailyStats] = useState<DailyStats[]>([]);
  const [weeklyStats, setWeeklyStats] = useState<WeeklyStats[]>([]);
  const [monthlyStats, setMonthlyStats] = useState<MonthlyStats[]>([]);
  const [totalStats, setTotalStats] = useState<TotalStats | null>(null);
  const [loading, setLoading] = useState(false);
  const [resetDialogOpen, setResetDialogOpen] = useState(false);
  const [resetLoading, setResetLoading] = useState(false);
  const [sessions, setSessions] = useState<WorkSession[]>([]);
  const [funStats, setFunStats] = useState<FunStats | null>(null);

  const loadStats = async () => {
    setLoading(true);
    try {
      console.log('🔄 开始加载统计数据...');
      
      const [daily, weekly, monthly, total, sessionsData, funStatsData] = await Promise.all([
        invoke<DailyStats[]>('get_daily_stats', { days: 30 }),
        invoke<WeeklyStats[]>('get_weekly_stats', { weeks: 12 }),
        invoke<MonthlyStats[]>('get_monthly_stats', { months: 12 }),
        invoke<[number, number, number]>('get_total_stats'),
        invoke<WorkSession[]>('get_work_sessions'),
        invoke<FunStats>('get_fun_stats').catch(error => {
          console.error('❌ get_fun_stats 调用失败:', error);
          return null;
        }),
      ]);

      console.log('📊 统计数据加载结果:', {
        daily: daily?.length || 0,
        weekly: weekly?.length || 0,
        monthly: monthly?.length || 0,
        total,
        sessions: sessionsData?.length || 0,
        funStats: funStatsData ? '成功' : '失败'
      });

      setDailyStats(daily);
      setWeeklyStats(weekly);
      setMonthlyStats(monthly);
      setTotalStats({
        total_steps: total[0],
        total_distance: total[1],
        total_days: total[2],
      });
      setSessions(sessionsData);
      setFunStats(funStatsData);
      
      console.log('✅ 统计数据加载完成');
    } catch (error) {
      console.error('❌ 加载统计数据失败:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleResetStats = async () => {
    setResetLoading(true);
    try {
      await invoke('clear_all_stats');
      setResetDialogOpen(false);
      
      // 发送重置事件通知主界面
      await emit('stats_reset', {});
      
      // 重新加载数据
      await loadStats();
      console.log('✅ 统计数据已清空，计步器已重置');
    } catch (error) {
      console.error('清空统计数据失败:', error);
      alert(`清空统计数据失败: ${error}`);
    } finally {
      setResetLoading(false);
    }
  };

  useEffect(() => {
    loadStats();
    
    // 每30秒自动刷新一次统计数据
    const interval = setInterval(() => {
      loadStats();
    }, 30000);
    
    return () => clearInterval(interval);
  }, []);

  const formatDistance = (distance: number) => {
    // 假设每100像素对应1米
    const meters = distance / 100.0;
    if (meters < 1000) {
      return `${meters.toFixed(1)}m`;
    }
    return `${(meters / 1000).toFixed(2)}km`;
  };

  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);
    return date.toLocaleDateString('zh-CN', {
      month: 'short',
      day: 'numeric',
    });
  };

  const formatWeek = (weekStr: string) => {
    const [year, week] = weekStr.split('-');
    return `${year}年第${week}周`;
  };

  const formatMonth = (monthStr: string) => {
    const [year, month] = monthStr.split('-');
    return `${year}年${month}月`;
  };

  const getProgressColor = (steps: number) => {
    if (steps >= 10000) return 'success';
    if (steps >= 5000) return 'warning';
    return 'error';
  };

  const renderDailyStats = () => (
    <TableContainer component={Paper} sx={{ maxHeight: 400 }}>
      <Table stickyHeader>
        <TableHead>
          <TableRow>
            <TableCell>日期</TableCell>
            <TableCell align="right">步数</TableCell>
            <TableCell align="right">距离</TableCell>
            <TableCell align="right">运动次数</TableCell>
            <TableCell align="center">进度</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {dailyStats.map((stat) => (
            <TableRow key={stat.id} hover>
              <TableCell>{formatDate(stat.date)}</TableCell>
              <TableCell align="right">
                <Typography variant="body2" fontWeight="bold">
                  {stat.total_steps.toLocaleString()}
                </Typography>
              </TableCell>
              <TableCell align="right">
                <Typography variant="body2" color="text.secondary">
                  {formatDistance(stat.total_distance)}
                </Typography>
              </TableCell>
              <TableCell align="right">
                <Typography variant="body2" color="text.secondary">
                  {stat.movement_count.toLocaleString()}
                </Typography>
              </TableCell>
              <TableCell align="center">
                <Box sx={{ width: '100%', mr: 1 }}>
                  <LinearProgress
                    variant="determinate"
                    value={Math.min((stat.total_steps / 10000) * 100, 100)}
                    color={getProgressColor(stat.total_steps) as any}
                    sx={{ height: 8, borderRadius: 4 }}
                  />
                </Box>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </TableContainer>
  );

  const renderWeeklyStats = () => (
    <TableContainer component={Paper} sx={{ maxHeight: 400 }}>
      <Table stickyHeader>
        <TableHead>
          <TableRow>
            <TableCell>周次</TableCell>
            <TableCell align="right">总步数</TableCell>
            <TableCell align="right">总距离</TableCell>
            <TableCell align="right">活跃天数</TableCell>
            <TableCell align="right">日均步数</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {weeklyStats.map((stat) => (
            <TableRow key={stat.id} hover>
              <TableCell>{formatWeek(stat.year_week)}</TableCell>
              <TableCell align="right">
                <Typography variant="body2" fontWeight="bold">
                  {stat.total_steps.toLocaleString()}
                </Typography>
              </TableCell>
              <TableCell align="right">
                <Typography variant="body2" color="text.secondary">
                  {formatDistance(stat.total_distance)}
                </Typography>
              </TableCell>
              <TableCell align="right">
                <Chip
                  label={stat.total_days}
                  size="small"
                  color={stat.total_days >= 5 ? 'success' : 'default'}
                />
              </TableCell>
              <TableCell align="right">
                <Typography variant="body2" fontWeight="bold">
                  {Math.round(stat.avg_steps_per_day).toLocaleString()}
                </Typography>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </TableContainer>
  );

  const renderMonthlyStats = () => (
    <TableContainer component={Paper} sx={{ maxHeight: 400 }}>
      <Table stickyHeader>
        <TableHead>
          <TableRow>
            <TableCell>月份</TableCell>
            <TableCell align="right">总步数</TableCell>
            <TableCell align="right">总距离</TableCell>
            <TableCell align="right">活跃天数</TableCell>
            <TableCell align="right">日均步数</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {monthlyStats.map((stat) => (
            <TableRow key={stat.id} hover>
              <TableCell>{formatMonth(stat.year_month)}</TableCell>
              <TableCell align="right">
                <Typography variant="body2" fontWeight="bold">
                  {stat.total_steps.toLocaleString()}
                </Typography>
              </TableCell>
              <TableCell align="right">
                <Typography variant="body2" color="text.secondary">
                  {formatDistance(stat.total_distance)}
                </Typography>
              </TableCell>
              <TableCell align="right">
                <Chip
                  label={stat.total_days}
                  size="small"
                  color={stat.total_days >= 20 ? 'success' : 'default'}
                />
              </TableCell>
              <TableCell align="right">
                <Typography variant="body2" fontWeight="bold">
                  {Math.round(stat.avg_steps_per_day).toLocaleString()}
                </Typography>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </TableContainer>
  );

  const renderWorkSessions = () => (
    <Box>
      {/* 牛马计时器说明 */}
      <Box sx={{ mb: 2, p: 2, bgcolor: 'background.paper', borderRadius: 1, border: 1, borderColor: 'divider' }}>
        <Typography variant="body2" color="text.secondary">
          🐮 牛马计时器说明：当您开始移动鼠标时，系统会自动开始记录牛马计时。
          如果15分钟内没有鼠标活动，会话将自动结束。
        </Typography>
      </Box>

      {/* 牛马计时器表格 */}
      <TableContainer component={Paper} sx={{ maxHeight: 400 }}>
        <Table stickyHeader>
          <TableHead>
            <TableRow>
              <TableCell>开始时间</TableCell>
              <TableCell>结束时间</TableCell>
              <TableCell align="right">时长</TableCell>
              <TableCell align="right">步数</TableCell>
              <TableCell align="right">距离</TableCell>
              <TableCell align="right">移动次数</TableCell>
              <TableCell align="center">状态</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {sessions.length === 0 ? (
              <TableRow>
                <TableCell colSpan={7} align="center" sx={{ py: 4 }}>
                  <Typography variant="body2" color="text.secondary">
                    暂无牛马计时记录
                  </Typography>
                </TableCell>
              </TableRow>
            ) : (
              sessions.map((session) => (
                <TableRow key={session.id} hover>
                  <TableCell>
                    <Typography variant="body2">
                      {new Date(session.start_time).toLocaleString('zh-CN', {
                        month: 'short',
                        day: 'numeric',
                        hour: '2-digit',
                        minute: '2-digit',
                      })}
                    </Typography>
                  </TableCell>
                  <TableCell>
                    <Typography variant="body2">
                      {session.end_time ? new Date(session.end_time).toLocaleString('zh-CN', {
                        month: 'short',
                        day: 'numeric',
                        hour: '2-digit',
                        minute: '2-digit',
                      }) : '-'}
                    </Typography>
                  </TableCell>
                  <TableCell align="right">
                    <Typography variant="body2" fontWeight="bold">
                      {Math.floor(session.duration_seconds / 60)}分{session.duration_seconds % 60}秒
                    </Typography>
                  </TableCell>
                  <TableCell align="right">
                    <Typography variant="body2">
                      {session.total_steps.toLocaleString()}
                    </Typography>
                  </TableCell>
                  <TableCell align="right">
                    <Typography variant="body2" color="text.secondary">
                      {formatDistance(session.total_distance)}
                    </Typography>
                  </TableCell>
                  <TableCell align="right">
                    <Typography variant="body2" color="text.secondary">
                      {session.movement_count.toLocaleString()}
                    </Typography>
                  </TableCell>
                  <TableCell align="center">
                    {session.end_time ? (
                      <Chip label="已完成" color="success" size="small" />
                    ) : (
                      <Chip label="进行中" color="primary" size="small" />
                    )}
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </TableContainer>
    </Box>
  );

  const renderFunStats = () => (
    <Box>
      {/* 趣味统计卡片 */}
      <Box sx={{ maxWidth: 600, mx: 'auto', mt: 2 }}>
        {!funStats ? (
          <Typography color="text.secondary">暂无数据</Typography>
        ) : (
          <Box sx={{ display: 'grid', gridTemplateColumns: { xs: '1fr', sm: '1fr 1fr' }, gap: 2 }}>
            <Card>
              <CardContent>
                <Typography variant="subtitle2" color="text.secondary">最长单次移动</Typography>
                <Typography variant="h5" fontWeight="bold">{funStats.longest_single_move.toFixed(1)} m</Typography>
              </CardContent>
            </Card>
            <Card>
              <CardContent>
                <Typography variant="subtitle2" color="text.secondary">最长工作时间</Typography>
                <Typography variant="h5" fontWeight="bold">{Math.floor(funStats.longest_work_session_seconds/60)}分{funStats.longest_work_session_seconds%60}秒</Typography>
              </CardContent>
            </Card>
            <Card>
              <CardContent>
                <Typography variant="subtitle2" color="text.secondary">单日最多步数</Typography>
                <Typography variant="h5" fontWeight="bold">{funStats.most_steps_in_day.toLocaleString()}</Typography>
              </CardContent>
            </Card>
            <Card>
              <CardContent>
                <Typography variant="subtitle2" color="text.secondary">单日最远距离</Typography>
                <Typography variant="h5" fontWeight="bold">{formatDistance(funStats.most_distance_in_day)}</Typography>
              </CardContent>
            </Card>
            <Card>
              <CardContent>
                <Typography variant="subtitle2" color="text.secondary">当前连续工作天数</Typography>
                <Typography variant="h5" fontWeight="bold">{funStats.current_work_streak}</Typography>
              </CardContent>
            </Card>
            <Card>
              <CardContent>
                <Typography variant="subtitle2" color="text.secondary">已解锁成就数</Typography>
                <Typography variant="h5" fontWeight="bold">{funStats.total_achievements_unlocked}</Typography>
              </CardContent>
            </Card>
            <Card>
              <CardContent>
                <Typography variant="subtitle2" color="text.secondary">最快移动速度</Typography>
                <Typography variant="h5" fontWeight="bold">{funStats.fastest_movement_speed.toFixed(1)} m/s</Typography>
              </CardContent>
            </Card>
            <Card>
              <CardContent>
                <Typography variant="subtitle2" color="text.secondary">牛马次数</Typography>
                <Typography variant="h5" fontWeight="bold">{funStats.total_work_sessions}</Typography>
              </CardContent>
            </Card>
          </Box>
        )}
      </Box>
      {/* 牛马计时器表格 */}
      <Box sx={{ mt: 4 }}>
        {renderWorkSessions()}
      </Box>
    </Box>
  );

  return (
    <Box sx={{ p: 3 }}>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 3 }}>
        <Typography variant="h5" fontWeight="bold" sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
          <ShowChart />
          运动统计
        </Typography>
        <Tooltip title="刷新数据">
          <IconButton onClick={loadStats} disabled={loading}>
            <Refresh />
          </IconButton>
        </Tooltip>
      </Box>

      {/* 总统计卡片 */}
      {totalStats && (
        <Box sx={{ display: 'grid', gridTemplateColumns: { xs: '1fr', md: 'repeat(3, 1fr)' }, gap: 2, mb: 3 }}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <TrendingUp color="primary" />
                <Typography variant="h6">总步数</Typography>
              </Box>
              <Typography variant="h4" fontWeight="bold" color="primary">
                {totalStats.total_steps.toLocaleString()}
              </Typography>
              <Typography variant="body2" color="text.secondary">
                累计运动步数
              </Typography>
            </CardContent>
          </Card>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <Timeline color="secondary" />
                <Typography variant="h6">总距离</Typography>
              </Box>
              <Typography variant="h4" fontWeight="bold" color="secondary">
                {formatDistance(totalStats.total_distance)}
              </Typography>
              <Typography variant="body2" color="text.secondary">
                累计移动距离
              </Typography>
            </CardContent>
          </Card>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <CalendarToday color="success" />
                <Typography variant="h6">活跃天数</Typography>
              </Box>
              <Typography variant="h4" fontWeight="bold" color="success.main">
                {totalStats.total_days}
              </Typography>
              <Typography variant="body2" color="text.secondary">
                有运动记录的天数
              </Typography>
            </CardContent>
          </Card>
        </Box>
      )}

      {/* 标签页 */}
      <Box sx={{ borderBottom: 1, borderColor: 'divider', mb: 2 }}>
        <Tabs value={Math.min(activeTab, 3)} onChange={(_, newValue) => setActiveTab(newValue)} variant="scrollable" scrollButtons="auto">
          <Tab label="每日" />
          <Tab label="每周" />
          <Tab label="每月" />
          <Tab label="趣味" />
        </Tabs>
      </Box>

      {/* 内容区域 */}
      <Box sx={{ mt: 2 }}>
        {activeTab === 0 && renderDailyStats()}
        {activeTab === 1 && renderWeeklyStats()}
        {activeTab === 2 && renderMonthlyStats()}
        {activeTab === 3 && renderFunStats()}
      </Box>

      {loading && (
        <Box sx={{ width: '100%', mt: 2 }}>
          <LinearProgress />
        </Box>
      )}

      {/* 重置按钮 */}
      <Box sx={{ mt: 4, pt: 3, borderTop: 1, borderColor: 'divider' }}>
        <Button
          variant="outlined"
          color="error"
          startIcon={<DeleteForever />}
          onClick={() => setResetDialogOpen(true)}
          fullWidth
          sx={{ py: 1.5 }}
        >
          清空所有统计数据
        </Button>
      </Box>

      {/* 重置确认对话框 */}
      <Dialog
        open={resetDialogOpen}
        onClose={() => setResetDialogOpen(false)}
        maxWidth="sm"
        fullWidth
      >
        <DialogTitle sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
          <DeleteForever color="error" />
          确认清空统计数据
        </DialogTitle>
        <DialogContent>
          <Alert severity="warning" sx={{ mb: 2 }}>
            此操作将永久删除所有历史统计数据，包括：
          </Alert>
          <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
            • 所有鼠标运动记录<br/>
            • 每日统计数据<br/>
            • 每周统计数据<br/>
            • 每月统计数据
          </Typography>
          <Typography variant="body2" color="error" fontWeight="bold">
            此操作不可撤销，请谨慎操作！
          </Typography>
        </DialogContent>
        <DialogActions>
          <Button 
            onClick={() => setResetDialogOpen(false)}
            disabled={resetLoading}
          >
            取消
          </Button>
          <Button 
            onClick={handleResetStats}
            color="error"
            variant="contained"
            disabled={resetLoading}
            startIcon={resetLoading ? <Refresh /> : <DeleteForever />}
          >
            {resetLoading ? '清空中...' : '确认清空'}
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
} 