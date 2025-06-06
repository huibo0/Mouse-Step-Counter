import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/tauri';
import {
  Box,
  Card,
  CardContent,
  Typography,
  LinearProgress,
  Chip,
  IconButton,
  Stack,
  Avatar,
  Fade,
  Grow,
} from '@mui/material';
import {
  Mouse,
  DirectionsWalk,
  RestartAlt,
  Timeline,
  TrendingUp,
  BugReport,
  Pets,
} from '@mui/icons-material';

function App() {
  const [steps, setSteps] = useState(0);
  const [lastSteps, setLastSteps] = useState(0);
  const [isIncreasing, setIsIncreasing] = useState(false);

  useEffect(() => {
    const unlisten = listen<number>('step_update', (event) => {
      const newSteps = event.payload;
      if (newSteps > steps) {
        setIsIncreasing(true);
        setTimeout(() => setIsIncreasing(false), 300);
      }
      setLastSteps(steps);
      setSteps(newSteps);
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, [steps]);

  useEffect(() => {
    // 根据步数自动调整狗子的奔跑速度
    const speed = Math.min(1 + (steps / 1000) * 2, 5); // 速度范围 1x - 5x
    // 在控制台打印狗子速度
    console.log(`🐕 狗子速度更新: ${speed.toFixed(1)}x (步数: ${steps})`);
  }, [steps]);

  const handleReset = async () => {
    try {
      await invoke('reset_counter');
      setSteps(0);
      setLastSteps(0);
    } catch (error) {
      console.error('重置失败:', error);
    }
  };

  const handleOpenDevTools = async () => {
    try {
      console.log('🐛 点击了调试按钮，正在打开开发者工具...');
      await invoke('open_devtools');
      console.log('✅ 开发者工具命令已发送');
      // 给用户一个视觉反馈
      alert('开发者工具已打开（可能是独立窗口）');
    } catch (error) {
      console.error('打开开发者工具失败:', error);
      alert(`打开开发者工具失败: ${error}`);
    }
  };

  const handleShowPetWindow = async () => {
    try {
      console.log('🐕 点击了显示宠物狗按钮，正在切换到宠物窗口...');
      await invoke('switch_to_pet_window');
      console.log('✅ 宠物窗口命令已发送');
    } catch (error) {
      console.error('显示宠物窗口失败:', error);
      alert(`显示宠物窗口失败: ${error}`);
    }
  };


  const progress = Math.min((steps % 1000) / 10, 100);
  const distance = (steps * 0.1).toFixed(1); // 假设每步0.1米

  return (
    <Box
      sx={{
        minHeight: '100vh',
        background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        padding: 2,
        fontFamily: 'system-ui, -apple-system, sans-serif',
        position: 'relative',
      }}
    >
      <Fade in timeout={1000}>
        <Card
          elevation={24}
          sx={{
            maxWidth: 400,
            width: '100%',
            borderRadius: 4,
            background: 'rgba(255, 255, 255, 0.95)',
            backdropFilter: 'blur(20px)',
            border: '1px solid rgba(255, 255, 255, 0.2)',
            position: 'relative',
            zIndex: 1,
          }}
        >
          <CardContent sx={{ padding: 4 }}>
            {/* Header */}
            <Stack direction="row" alignItems="center" spacing={2} mb={3}>
              <Avatar
                sx={{
                  background: 'linear-gradient(45deg, #667eea, #764ba2)',
                  width: 56,
                  height: 56,
                }}
              >
                <Mouse sx={{ fontSize: 28 }} />
              </Avatar>
              <Box sx={{ flex: 1 }}>
                <Typography variant="h5" fontWeight="bold" color="text.primary">
                  鼠标计步器
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  Mouse Step Counter
                </Typography>
              </Box>

            </Stack>

            {/* Main Counter */}
            <Box textAlign="center" mb={4}>
              <Grow in timeout={800}>
                <Box>
                  <Typography
                    variant="h2"
                    fontWeight="bold"
                    sx={{
                      background: 'linear-gradient(45deg, #667eea, #764ba2)',
                      WebkitBackgroundClip: 'text',
                      WebkitTextFillColor: 'transparent',
                      transform: isIncreasing ? 'scale(1.1)' : 'scale(1)',
                      transition: 'transform 0.3s ease',
                    }}
                  >
                    {steps.toLocaleString()}
                  </Typography>
                  <Typography variant="h6" color="text.secondary" gutterBottom>
                    步数
                  </Typography>
                </Box>
              </Grow>
            </Box>

            {/* Progress Bar */}
            <Box mb={3}>
              <Stack direction="row" justifyContent="space-between" mb={1}>
                <Typography variant="body2" color="text.secondary">
                  进度到下个千步
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  {progress.toFixed(0)}%
                </Typography>
              </Stack>
              <LinearProgress
                variant="determinate"
                value={progress}
                sx={{
                  height: 8,
                  borderRadius: 4,
                  backgroundColor: 'rgba(103, 126, 234, 0.1)',
                  '& .MuiLinearProgress-bar': {
                    background: 'linear-gradient(90deg, #667eea, #764ba2)',
                    borderRadius: 4,
                  },
                }}
              />
            </Box>

            {/* Stats Cards */}
            <Stack direction="row" spacing={2} mb={3}>
              <Card
                variant="outlined"
                sx={{
                  flex: 1,
                  background: 'rgba(103, 126, 234, 0.05)',
                  borderColor: 'rgba(103, 126, 234, 0.2)',
                }}
              >
                <CardContent sx={{ padding: 2, '&:last-child': { pb: 2 } }}>
                  <Stack direction="row" alignItems="center" spacing={1}>
                    <DirectionsWalk color="primary" />
                    <Box>
                      <Typography variant="h6" fontWeight="bold">
                        {distance}m
                      </Typography>
                      <Typography variant="caption" color="text.secondary">
                        距离
                      </Typography>
                    </Box>
                  </Stack>
                </CardContent>
              </Card>

              <Card
                variant="outlined"
                sx={{
                  flex: 1,
                  background: 'rgba(118, 75, 162, 0.05)',
                  borderColor: 'rgba(118, 75, 162, 0.2)',
                }}
              >
                <CardContent sx={{ padding: 2, '&:last-child': { pb: 2 } }}>
                  <Stack direction="row" alignItems="center" spacing={1}>
                    <TrendingUp color="secondary" />
                    <Box>
                      <Typography variant="h6" fontWeight="bold">
                        {steps > lastSteps ? '+' : ''}{steps - lastSteps}
                      </Typography>
                      <Typography variant="caption" color="text.secondary">
                        增量
                      </Typography>
                    </Box>
                  </Stack>
                </CardContent>
              </Card>
            </Stack>

            {/* Status Chip */}
            <Box display="flex" justifyContent="center" mb={2}>
              <Chip
                icon={<Timeline />}
                label={steps > 0 ? "监听中 🖱️" : "等待鼠标移动..."}
                color={steps > 0 ? "success" : "default"}
                variant="outlined"
                sx={{
                  borderRadius: 3,
                  fontWeight: 'medium',
                }}
              />
            </Box>

            {/* Action Buttons */}
            <Stack direction="row" justifyContent="center" spacing={2}>
              <IconButton
                onClick={handleReset}
                size="large"
                sx={{
                  background: 'rgba(103, 126, 234, 0.1)',
                  '&:hover': {
                    background: 'rgba(103, 126, 234, 0.2)',
                  },
                }}
                title="重置计数器"
              >
                <RestartAlt />
              </IconButton>
              
              <IconButton
                onClick={(e) => {
                  console.log('🖱️ 宠物狗按钮被点击了！');
                  e.preventDefault();
                  e.stopPropagation();
                  handleShowPetWindow();
                }}
                size="large"
                sx={{
                  background: 'rgba(76, 175, 80, 0.1)',
                  '&:hover': {
                    background: 'rgba(76, 175, 80, 0.2)',
                  },
                }}
                title="显示宠物狗"
              >
                <Pets />
              </IconButton>
              
              <IconButton
                onClick={(e) => {
                  console.log('🖱️ 调试按钮被点击了！');
                  e.preventDefault();
                  e.stopPropagation();
                  handleOpenDevTools();
                }}
                size="large"
                sx={{
                  background: 'rgba(255, 152, 0, 0.1)',
                  '&:hover': {
                    background: 'rgba(255, 152, 0, 0.2)',
                  },
                }}
                title="打开开发者工具"
              >
                <BugReport />
              </IconButton>
            </Stack>
          </CardContent>
        </Card>
      </Fade>
    </Box>
  );
}

export default App; 