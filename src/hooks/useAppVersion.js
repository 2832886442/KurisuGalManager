import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

let cachedVersion = null;

export default function useAppVersion() {
  const [version, setVersion] = useState(cachedVersion ?? '');
  useEffect(() => {
    if (cachedVersion) return;
    invoke('get_app_version')
      .then((v) => { cachedVersion = v; setVersion(v); })
      .catch(() => { cachedVersion = '未知'; setVersion('未知'); });
  }, []);
  return version;
}
