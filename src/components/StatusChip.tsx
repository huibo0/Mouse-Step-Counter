import { Box, Chip } from '@mui/material';
import { Timeline } from '@mui/icons-material';

interface StatusChipProps {
  steps: number;
}

export function StatusChip({ steps }: StatusChipProps) {
  return (
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
  );
} 