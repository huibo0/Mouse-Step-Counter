import { Card, CardContent, Stack, Typography, Box, Chip, Button, Alert } from '@mui/material';
import { LocalDrink, Warning } from '@mui/icons-material';
import { WaterReminderConfig, WaterReminderState } from '../types/water';
import { invoke } from '@tauri-apps/api/tauri';

interface WaterReminderCardProps {
  waterState: WaterReminderState | null;
  waterConfig: WaterReminderConfig;
  onWaterDrunk?: () => void;
}

export function WaterReminderCard({ waterState, waterConfig, onWaterDrunk }: WaterReminderCardProps) {
  if (!waterState || !waterConfig.enabled) return null;

  // UI应该直接反映后端传来的状态，而不是在前端重新计算
  // 后端已经判断好当前时间段是否需要提醒
  const isOverdue = !waterState.current_period_water_drunk;
  
  const progress = (waterState.glasses_drunk_today / waterConfig.daily_glasses) * 100;

  const handleDrinkWater = async () => {
    try {
      await invoke('record_water_drunk');
      console.log('💧 记录喝水成功');
      if (onWaterDrunk) {
        onWaterDrunk();
      }
    } catch (error) {
      console.error('记录喝水失败:', error);
    }
  };

  return (
    <Card
      variant="outlined"
      sx={{
        mb: 3,
        background: isOverdue 
          ? 'rgba(255, 152, 0, 0.05)' 
          : 'rgba(33, 150, 243, 0.05)',
        borderColor: isOverdue 
          ? 'rgba(255, 152, 0, 0.3)' 
          : 'rgba(33, 150, 243, 0.2)',
        borderWidth: isOverdue ? 2 : 1,
        position: 'relative',
        overflow: 'visible',
      }}
    >
      {isOverdue && (
        <Box
          sx={{
            position: 'absolute',
            top: -8,
            right: -8,
            background: 'linear-gradient(45deg, #ff9800, #ff5722)',
            borderRadius: '50%',
            width: 24,
            height: 24,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            boxShadow: '0 2px 8px rgba(255, 152, 0, 0.4)',
            animation: 'pulse 2s infinite',
            '@keyframes pulse': {
              '0%': {
                transform: 'scale(1)',
                boxShadow: '0 2px 8px rgba(255, 152, 0, 0.4)',
              },
              '50%': {
                transform: 'scale(1.1)',
                boxShadow: '0 4px 16px rgba(255, 152, 0, 0.6)',
              },
              '100%': {
                transform: 'scale(1)',
                boxShadow: '0 2px 8px rgba(255, 152, 0, 0.4)',
              },
            },
          }}
        >
          <Warning sx={{ fontSize: 14, color: 'white' }} />
        </Box>
      )}

      <CardContent sx={{ padding: 2, '&:last-child': { pb: 2 } }}>
        <Stack spacing={2}>
          <Stack direction="row" alignItems="center" spacing={1}>
            <LocalDrink color={isOverdue ? "warning" : "primary"} />
            <Box sx={{ flex: 1 }}>
              <Typography variant="h6" fontWeight="bold">
                {waterState.glasses_drunk_today}/{waterConfig.daily_glasses}
              </Typography>
              <Typography variant="caption" color="text.secondary">
                今日喝水
              </Typography>
            </Box>
            <Chip
              size="small"
              label={waterConfig.enabled ? "已开启" : "已关闭"}
              color={waterConfig.enabled ? "success" : "default"}
              variant="outlined"
            />
          </Stack>

          {isOverdue && (
            <Alert 
              severity="warning" 
              icon={<LocalDrink />}
              sx={{ 
                py: 0.5,
                '& .MuiAlert-message': {
                  py: 0.5,
                }
              }}
            >
              <Stack direction="row" alignItems="center" justifyContent="space-between" spacing={2} sx={{ width: '100%' }}>
                <Typography variant="body2" sx={{ flexGrow: 1, textAlign: 'left' }}>
                  该喝水了！还差 {waterConfig.daily_glasses - waterState.glasses_drunk_today} 杯
                </Typography>
                <Button
                  variant="contained"
                  size="small"
                  startIcon={<LocalDrink />}
                  onClick={handleDrinkWater}
                  sx={{
                    background: 'linear-gradient(45deg, #ff9800, #ff5722)',
                    '&:hover': {
                      background: 'linear-gradient(45deg, #f57c00, #e64a19)',
                    },
                    minWidth: 80,
                  }}
                >
                  喝水
                </Button>
              </Stack>
            </Alert>
          )}

          {/* 进度条 */}
          <Box>
            <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 0.5 }}>
              <Typography variant="caption" color="text.secondary">
                进度
              </Typography>
              <Typography variant="caption" color="text.secondary">
                {progress.toFixed(0)}%
              </Typography>
            </Box>
            <Box
              sx={{
                width: '100%',
                height: 4,
                backgroundColor: 'rgba(0, 0, 0, 0.1)',
                borderRadius: 2,
                overflow: 'hidden',
              }}
            >
              <Box
                sx={{
                  width: `${progress}%`,
                  height: '100%',
                  background: isOverdue 
                    ? 'linear-gradient(90deg, #ff9800, #ff5722)'
                    : 'linear-gradient(90deg, #2196f3, #1976d2)',
                  borderRadius: 2,
                  transition: 'width 0.3s ease',
                }}
              />
            </Box>
          </Box>
        </Stack>
      </CardContent>
    </Card>
  );
} 