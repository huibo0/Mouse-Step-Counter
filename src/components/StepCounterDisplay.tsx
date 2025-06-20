import { Box, Typography, LinearProgress, Stack } from '@mui/material';
import { Grow } from '@mui/material';

interface StepCounterDisplayProps {
  steps: number;
  isIncreasing: boolean;
}

export function StepCounterDisplay({ steps, isIncreasing }: StepCounterDisplayProps) {
  const progress = Math.min((steps % 1000) / 10, 100);

  return (
    <>
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
    </>
  );
} 