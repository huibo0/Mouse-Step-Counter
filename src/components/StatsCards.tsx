import { Stack, Card, CardContent, Typography, Box } from '@mui/material';
import { DirectionsWalk, TrendingUp } from '@mui/icons-material';

interface StatsCardsProps {
  distance: string;
  steps: number;
  lastSteps: number;
}

export function StatsCards({ distance, steps, lastSteps }: StatsCardsProps) {
  return (
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
  );
} 