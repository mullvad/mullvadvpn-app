import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { useSelector } from '../../../../../redux/store';
import { AriaDescription, AriaInput, AriaInputGroup, AriaLabel } from '../../../../AriaGroup';
import * as Cell from '../../../../cell';

export function AnimateMapSetting() {
  const animateMap = useSelector((state) => state.settings.guiSettings.animateMap);
  const { setAnimateMap } = useAppContext();

  return (
    <AriaInputGroup>
      <Cell.Container>
        <AriaLabel>
          <Cell.InputLabel>
            {messages.pgettext('user-interface-settings-view', 'Animate map')}
          </Cell.InputLabel>
        </AriaLabel>
        <AriaInput>
          <Cell.Switch isOn={animateMap} onChange={setAnimateMap} />
        </AriaInput>
      </Cell.Container>
      <Cell.CellFooter>
        <AriaDescription>
          <Cell.CellFooterText>
            {messages.pgettext('user-interface-settings-view', 'Animate map movements.')}
          </Cell.CellFooterText>
        </AriaDescription>
      </Cell.CellFooter>
    </AriaInputGroup>
  );
}
