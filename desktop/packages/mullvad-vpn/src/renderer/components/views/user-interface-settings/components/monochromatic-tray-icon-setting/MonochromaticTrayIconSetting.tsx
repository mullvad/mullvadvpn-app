import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { useSelector } from '../../../../../redux/store';
import { AriaDescription, AriaInput, AriaInputGroup, AriaLabel } from '../../../../AriaGroup';
import * as Cell from '../../../../cell';

export function MonochromaticTrayIconSetting() {
  const monochromaticIcon = useSelector((state) => state.settings.guiSettings.monochromaticIcon);
  const { setMonochromaticIcon } = useAppContext();

  return (
    <AriaInputGroup>
      <Cell.Container>
        <AriaLabel>
          <Cell.InputLabel>
            {messages.pgettext('user-interface-settings-view', 'Monochromatic tray icon')}
          </Cell.InputLabel>
        </AriaLabel>
        <AriaInput>
          <Cell.Switch isOn={monochromaticIcon} onChange={setMonochromaticIcon} />
        </AriaInput>
      </Cell.Container>
      <Cell.CellFooter>
        <AriaDescription>
          <Cell.CellFooterText>
            {messages.pgettext(
              'user-interface-settings-view',
              'Use a monochromatic tray icon instead of a colored one.',
            )}
          </Cell.CellFooterText>
        </AriaDescription>
      </Cell.CellFooter>
    </AriaInputGroup>
  );
}
