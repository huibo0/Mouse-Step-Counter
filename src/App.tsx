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
  Pets,
} from '@mui/icons-material';

function App() {
  const [steps, setSteps] = useState(0);
  const [lastSteps, setLastSteps] = useState(0);
  const [isIncreasing, setIsIncreasing] = useState(false);
  const [dogSpeed, setDogSpeed] = useState(1);

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
    setDogSpeed(speed);
  }, [steps]);

  const handleReset = async () => {
    try {
      await invoke('reset_counter');
      setSteps(0);
      setLastSteps(0);
      setDogSpeed(1);
    } catch (error) {
      console.error('重置失败:', error);
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
              <Box>
                <Typography variant="h5" fontWeight="bold" color="text.primary">
                  鼠标计步器
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  Mouse Step Counter
                </Typography>
              </Box>
            </Stack>

            {/* 小狗速度显示 */}
            <Box textAlign="center" mb={2}>
              <Chip
                icon={<Pets />}
                label={`狗子速度: ${dogSpeed.toFixed(1)}x`}
                color="primary"
                variant="outlined"
                size="small"
                sx={{
                  borderRadius: 3,
                  fontWeight: 'medium',
                  background: 'rgba(103, 126, 234, 0.05)',
                }}
              />
            </Box>

            {/* Main Counter */}
            <Box textAlign="center" mb={4} position="relative">
              <Grow in timeout={800}>
                <Box>
                  <Stack direction="row" alignItems="center" justifyContent="center" spacing={2}>
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
                    
                    {/* 奔跑的小狗 - 固定在数字旁边 */}
                    <Box
                      sx={{
                        width: '60px',
                        height: '45px',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        opacity: steps > 0 ? 1 : 0.3,
                        transition: 'opacity 0.5s ease',
                        animation: `bounce ${1 / dogSpeed}s ease-in-out infinite`,
                        '@keyframes bounce': {
                          '0%, 100%': { transform: 'translateY(0px)' },
                          '50%': { transform: 'translateY(-3px)' },
                        },
                      }}
                    >
                      <img
                        src="/img/running_dog.gif"
                        alt="Running Dog"
                        style={{
                          width: '60px',
                          height: '45px',
                          filter: 'drop-shadow(2px 2px 4px rgba(0,0,0,0.1))',
                        }}
                        onLoad={() => console.log('Dog image loaded successfully')}
                        onError={(e) => {
                          console.error('Failed to load dog image:', e);
                          e.currentTarget.style.display = 'none';
                          const fallback = e.currentTarget.parentElement?.querySelector('.dog-fallback') as HTMLElement;
                          if (fallback) fallback.style.display = 'flex';
                        }}
                      />
                      <Box
                        className="dog-fallback"
                        sx={{
                          width: '60px',
                          height: '45px',
                          display: 'none',
                          alignItems: 'center',
                          justifyContent: 'center',
                          fontSize: '36px',
                          filter: 'drop-shadow(2px 2px 4px rgba(0,0,0,0.1))',
                        }}
                      >
                        🐕
                      </Box>
                    </Box>
                  </Stack>
                  
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

            {/* Reset Button */}
            <Box display="flex" justifyContent="center">
              <IconButton
                onClick={handleReset}
                size="large"
                sx={{
                  background: 'rgba(103, 126, 234, 0.1)',
                  '&:hover': {
                    background: 'rgba(103, 126, 234, 0.2)',
                  },
                }}
              >
                <RestartAlt />
              </IconButton>
            </Box>
          </CardContent>
        </Card>
      </Fade>
    </Box>
  );
}

export default App; 