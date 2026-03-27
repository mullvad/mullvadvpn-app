import { useSelector } from '../../../redux/store';

export function useObfuscation() {
  const obfuscation = useSelector(
    (state) => state.settings.obfuscationSettings.selectedObfuscation,
  );

  return { obfuscation };
}
