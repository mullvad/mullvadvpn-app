import { useCallback } from 'react';
import styled from 'styled-components';

import { messages } from '../../shared/gettext';
import { useAppContext } from '../context';
import { useBoolean } from '../lib/utilityHooks';
import { useSelector } from '../redux/store';
import * as AppButton from './AppButton';
import { hugeText, smallText } from './common-styles';
import { ModalAlert, ModalMessage } from './Modal';

const StyledTitle = styled.h1(hugeText, {
  textAlign: 'center',
  margin: '7px 0 4px',
});

const StyledSubTitle = styled.span(smallText, {
  marginTop: '10px',
  fontWeight: 700,
});

const StyledList = styled.ul({
  listStyle: 'disc outside',
  marginLeft: '20px',
});

const StyledMessage = styled(ModalMessage)({
  fontSize: '12px',
  marginTop: '6px',
});

export function Changelog() {
  const currentVersion = useSelector((state) => state.version.current);
  const changelogDisplayedForVersion = useSelector(
    (state) => state.settings.guiSettings.changelogDisplayedForVersion,
  );
  const changelog = useSelector((state) => state.userInterface.changelog);
  const initialForceShowChanges = useSelector((state) => state.userInterface.forceShowChanges);

  const { setDisplayedChangelog } = useAppContext();

  const [forceShowChanges, , stopForceShowChanges] = useBoolean(initialForceShowChanges);

  const close = useCallback(() => {
    setDisplayedChangelog();
    stopForceShowChanges();
  }, []);

  const visible =
    forceShowChanges ||
    (changelogDisplayedForVersion !== currentVersion &&
      changelog.length > 0 &&
      !window.env.development &&
      !/-dev-[0-9a-f]{6}$/.test(currentVersion));

  return (
    <ModalAlert
      isOpen={visible}
      buttons={[
        <AppButton.BlueButton key="close" onClick={close}>
          {
            // TRANSLATORS: This is a button which closes a dialog.
            messages.gettext('Got it!')
          }
        </AppButton.BlueButton>,
      ]}>
      <StyledTitle>{currentVersion}</StyledTitle>
      <StyledSubTitle>{messages.pgettext('changelog', 'Changes in this version:')}</StyledSubTitle>
      <StyledMessage>
        <StyledList>
          {changelog.map((item, i) => (
            <li key={i}>{item}</li>
          ))}
        </StyledList>
      </StyledMessage>
    </ModalAlert>
  );
}
