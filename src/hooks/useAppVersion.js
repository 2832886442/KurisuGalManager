import { useState, useEffect } from 'react';
import { getVersion } from '@tauri-apps/api/app';

let cachedVersion = null;

export default function useAppVersion() {
  const [version, setVersion] = useState(cachedVersion ?? '');
  useEffect(() => {
    if (cachedVersion) return;
    getVersion()
      .then((v) => { cachedVersion = v; setVersion(v); })
      .catch(() => { cachedVersion = '未知'; setVersion('未知'); });
  }, []);
  return version;
}
