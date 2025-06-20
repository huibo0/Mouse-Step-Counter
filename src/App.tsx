import { useEffect, useState, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/tauri';
import {
  Box,
  Card,
  CardContent,
  Fade,
} from '@mui/material';

import { HeaderBar } from './components/HeaderBar';
import { StepCounterDisplay } from './components/StepCounterDisplay';
import { StatsCards } from './components/StatsCards';
import { WaterReminderCard } from './components/WaterReminderCard';
import { StatusChip } from './components/StatusChip';
import { ActionButtons } from './components/ActionButtons';
import { WaterReminderConfigDialog } from './components/WaterReminderConfigDialog';
import { WaterReminderConfig, WaterReminderState } from './types/water';

function App() {
  const [steps, setSteps] = useState(0);
  const [lastSteps, setLastSteps] = useState(0);
  const [isIncreasing, setIsIncreasing] = useState(false);
  const [waterConfig, setWaterConfig] = useState<WaterReminderConfig>({
    daily_glasses: 8,
    reminder_interval_hours: 1,
    custom_reminder_times: [],
    enabled: false,
  });
  const [waterState, setWaterState] = useState<WaterReminderState | null>(null);
  const [configDialogOpen, setConfigDialogOpen] = useState(false);
  const [tempConfig, setTempConfig] = useState<WaterReminderConfig>(waterConfig);

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

  // 加载喝水提醒配置和状态
  useEffect(() => {
    const loadWaterData = async () => {
      try {
        const config = await invoke<WaterReminderConfig>('get_water_reminder_config');
        const state = await invoke<WaterReminderState>('get_water_reminder_state');
        setWaterConfig(config);
        setWaterState(state);
        setTempConfig(config);
      } catch (error) {
        console.error('加载喝水提醒数据失败:', error);
      }
    };
    loadWaterData();
  }, []);

  // 刷新喝水状态
  const refreshWaterState = useCallback(async () => {
    try {
      console.log('🔄 [APP] Refreshing water state...');
      const state = await invoke<WaterReminderState>('get_water_reminder_state');
      setWaterState(state);
      console.log('✅ [APP] Water state refreshed:', state);
    } catch (error) {
      console.error('❌ [APP] Failed to refresh water state:', error);
    }
  }, []);

  // 监听后端发来的喝水提醒事件
  useEffect(() => {
    const unlisten = listen<string>('water_reminder', (event) => {
      console.log('✅✅✅ [APP] Received water_reminder event!', event.payload);
      refreshWaterState();
    });

    return () => {
      console.log('[APP] Unregistering water_reminder listener.');
      unlisten.then((f) => f());
    };
  }, [refreshWaterState]);

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
      // alert('开发者工具已打开（可能是独立窗口）');
    } catch (error) {
      console.error('打开开发者工具失败:', error);
      // alert(`打开开发者工具失败: ${error}`);
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

  const handleOpenWaterConfig = () => {
    setTempConfig(waterConfig);
    setConfigDialogOpen(true);
  };

  const handleSaveWaterConfig = async () => {
    try {
      await invoke('update_water_reminder_config', { config: tempConfig });
      setWaterConfig(tempConfig);
      setConfigDialogOpen(false);
      console.log('💧 喝水提醒配置已保存');
    } catch (error) {
      console.error('保存喝水提醒配置失败:', error);
      alert(`保存配置失败: ${error}`);
    }
  };

  const handleCancelWaterConfig = () => {
    setTempConfig(waterConfig);
    setConfigDialogOpen(false);
  };

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
            <HeaderBar />
            
            <StepCounterDisplay 
              steps={steps} 
              isIncreasing={isIncreasing} 
            />

            <StatsCards 
              distance={distance}
              steps={steps}
              lastSteps={lastSteps}
            />

            <WaterReminderCard 
              waterState={waterState}
              waterConfig={waterConfig}
              onWaterDrunk={refreshWaterState}
            />

            <StatusChip steps={steps} />

            <ActionButtons
              onReset={handleReset}
              onShowPet={handleShowPetWindow}
              onOpenWaterConfig={handleOpenWaterConfig}
              onOpenDevTools={handleOpenDevTools}
            />
          </CardContent>
        </Card>
      </Fade>

      <WaterReminderConfigDialog
        open={configDialogOpen}
        config={tempConfig}
        onClose={handleCancelWaterConfig}
        onSave={handleSaveWaterConfig}
        onConfigChange={setTempConfig}
      />
    </Box>
  );
}

export default App; 