import { Stack, Avatar, Box, Typography } from '@mui/material';
import { Mouse } from '@mui/icons-material';

interface HeaderBarProps {
  title?: string;
  subtitle?: string;
}

export function HeaderBar({ 
  title = "鼠标计步器", 
  subtitle = "Mouse Step Counter" 
}: HeaderBarProps) {
  return (
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
          {title}
        </Typography>
        <Typography variant="body2" color="text.secondary">
          {subtitle}
        </Typography>
      </Box>
    </Stack>
  );
} 