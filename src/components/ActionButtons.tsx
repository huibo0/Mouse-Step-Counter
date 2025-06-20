import { Stack, IconButton } from '@mui/material';
import {
  RestartAlt,
  Pets,
  LocalDrink,
  BugReport,
} from '@mui/icons-material';

interface ActionButtonsProps {
  onReset: () => void;
  onShowPet: () => void;
  onOpenWaterConfig: () => void;
  onOpenDevTools: () => void;
}

export function ActionButtons({
  onReset,
  onShowPet,
  onOpenWaterConfig,
  onOpenDevTools,
}: ActionButtonsProps) {
  return (
    <Stack direction="row" justifyContent="center" spacing={2}>
      <IconButton
        onClick={onReset}
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
          onShowPet();
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
        onClick={onOpenWaterConfig}
        size="large"
        sx={{
          background: 'rgba(33, 150, 243, 0.1)',
          '&:hover': {
            background: 'rgba(33, 150, 243, 0.2)',
          },
        }}
        title="喝水提醒设置"
      >
        <LocalDrink />
      </IconButton>
      
      <IconButton
        onClick={(e) => {
          console.log('🖱️ 调试按钮被点击了！');
          e.preventDefault();
          e.stopPropagation();
          onOpenDevTools();
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
  );
} 