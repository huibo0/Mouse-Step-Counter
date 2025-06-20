import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  TextField,
  Switch,
  FormControlLabel,
  Divider,
  Alert,
  Stack,
  Box,
  IconButton,
  Typography,
} from '@mui/material';
import {
  LocalDrink,
  TrendingUp,
  RestartAlt,
} from '@mui/icons-material';
import { WaterReminderConfig } from '../types/water';

interface WaterReminderConfigDialogProps {
  open: boolean;
  config: WaterReminderConfig;
  onClose: () => void;
  onSave: () => void;
  onConfigChange: (config: WaterReminderConfig) => void;
}

export function WaterReminderConfigDialog({
  open,
  config,
  onClose,
  onSave,
  onConfigChange,
}: WaterReminderConfigDialogProps) {
  const addCustomTime = () => {
    onConfigChange({
      ...config,
      custom_reminder_times: [...config.custom_reminder_times, '09:00']
    });
  };

  const removeCustomTime = (index: number) => {
    onConfigChange({
      ...config,
      custom_reminder_times: config.custom_reminder_times.filter((_, i) => i !== index)
    });
  };

  const updateCustomTime = (index: number, value: string) => {
    onConfigChange({
      ...config,
      custom_reminder_times: config.custom_reminder_times.map((time, i) => i === index ? value : time)
    });
  };

  return (
    <Dialog 
      open={open} 
      onClose={onClose}
      maxWidth="sm"
      fullWidth
    >
      <DialogTitle>
        <Stack direction="row" alignItems="center" spacing={1}>
          <LocalDrink color="primary" />
          <Typography variant="h6">喝水提醒设置</Typography>
        </Stack>
      </DialogTitle>
      <DialogContent>
        <Stack spacing={3} sx={{ mt: 1 }}>
          <FormControlLabel
            control={
              <Switch
                checked={config.enabled}
                onChange={(e) => onConfigChange({ ...config, enabled: e.target.checked })}
              />
            }
            label="启用喝水提醒"
          />

          <TextField
            label="每日目标杯数"
            type="number"
            value={config.daily_glasses}
            onChange={(e) => onConfigChange({ ...config, daily_glasses: parseInt(e.target.value) || 8 })}
            inputProps={{ min: 1, max: 20 }}
            fullWidth
          />

          <Divider />

          <Typography variant="subtitle1" fontWeight="bold">
            提醒方式
          </Typography>

          <Box>
            <FormControlLabel
              control={
                <Switch
                  checked={config.custom_reminder_times.length === 0}
                  onChange={(e) => {
                    if (e.target.checked) {
                      onConfigChange({ ...config, custom_reminder_times: [] });
                    }
                  }}
                />
              }
              label="按时间间隔提醒"
            />
            
            {config.custom_reminder_times.length === 0 && (
              <TextField
                label="提醒间隔（小时）"
                type="number"
                value={config.reminder_interval_hours}
                onChange={(e) => onConfigChange({ ...config, reminder_interval_hours: parseInt(e.target.value) || 1 })}
                inputProps={{ min: 1, max: 24 }}
                sx={{ mt: 1 }}
                fullWidth
              />
            )}
          </Box>

          <Box>
            <FormControlLabel
              control={
                <Switch
                  checked={config.custom_reminder_times.length > 0}
                  onChange={(e) => {
                    if (e.target.checked) {
                      onConfigChange({ ...config, custom_reminder_times: ['09:00'] });
                    } else {
                      onConfigChange({ ...config, custom_reminder_times: [] });
                    }
                  }}
                />
              }
              label="自定义提醒时间"
            />
            
            {config.custom_reminder_times.length > 0 && (
              <Stack spacing={1} sx={{ mt: 1 }}>
                {config.custom_reminder_times.map((time, index) => (
                  <Stack key={index} direction="row" spacing={1} alignItems="center">
                    <TextField
                      type="time"
                      value={time}
                      onChange={(e) => updateCustomTime(index, e.target.value)}
                      size="small"
                    />
                    <IconButton
                      size="small"
                      onClick={() => removeCustomTime(index)}
                      sx={{ color: 'error.main' }}
                    >
                      <RestartAlt />
                    </IconButton>
                  </Stack>
                ))}
                <Button
                  variant="outlined"
                  size="small"
                  onClick={addCustomTime}
                  startIcon={<TrendingUp />}
                >
                  添加时间
                </Button>
              </Stack>
            )}
          </Box>

          <Alert severity="info">
            💧 推荐每天喝8杯水，保持身体水分平衡。提醒会在指定时间弹出，点击"我喝了水"按钮记录。
          </Alert>
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>取消</Button>
        <Button onClick={onSave} variant="contained">
          保存设置
        </Button>
      </DialogActions>
    </Dialog>
  );
} 