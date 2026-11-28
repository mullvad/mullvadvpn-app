import { useCallback } from 'react';
import styled from 'styled-components';

import { messages } from '../../shared/gettext';
import log from '../../shared/logging';
import { useScheduler } from '../../shared/scheduler';
import { Flex, Icon, IconButton } from '../lib/components';
import { useBoolean } from '../lib/utility-hooks';

const COPIED_ICON_DURATION = 2000;

interface IProps extends React.HTMLAttributes<HTMLElement> {
  value: string;
  displayValue?: string;
  obscureValue?: boolean;
  message?: string;
  // Accessible name of the copy button.
  copyLabel?: string;
}

const StyledLabelContainer = styled.div({
  display: 'flex',
  flex: 1,
  minHeight: '19px',
  alignItems: 'center',
});

const StyledLabel = styled.span({
  flex: 1,
  minWidth: 0,
  // Long values such as WireGuard public keys have no natural break points.
  overflowWrap: 'anywhere',
});

export default function ClipboardLabel(props: IProps) {
  const { value, obscureValue, displayValue, message, copyLabel, ...otherProps } = props;

  const [obscured, , , toggleObscured] = useBoolean(obscureValue ?? true);
  const [justCopied, setJustCopied, resetJustCopied] = useBoolean(false);

  const copiedScheduler = useScheduler();

  const onCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(value);
      copiedScheduler.schedule(resetJustCopied, COPIED_ICON_DURATION);
      setJustCopied();
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to copy to clipboard: ${error.message}`);
    }
  }, [value, copiedScheduler, setJustCopied, resetJustCopied]);

  return (
    <StyledLabelContainer>
      <StyledLabel aria-hidden={obscured} {...otherProps}>
        {obscured ? '●●●● ●●●● ●●●● ●●●●' : (displayValue ?? value)}
      </StyledLabel>
      <Flex gap="medium">
        {obscureValue !== false && (
          <IconButton
            onClick={toggleObscured}
            aria-label={
              obscured
                ? // This line is here to prevent the following one to be moved up here by prettier
                  // TRANSLATORS: Provided to accessibility tools such as screenreaders to describe
                  // TRANSLATORS: the button which unobscures a hidden value.
                  messages.pgettext('accessibility', 'Show value')
                : // This line is here to prevent the following one to be moved up here by prettier
                  // TRANSLATORS: Provided to accessibility tools such as screenreaders to describe
                  // TRANSLATORS: the button which obscures a value.
                  messages.pgettext('accessibility', 'Hide value')
            }>
            {obscured ? <IconButton.Icon icon="show" /> : <IconButton.Icon icon="hide" />}
          </IconButton>
        )}
        {justCopied ? (
          <Icon icon="checkmark" color="green"></Icon>
        ) : (
          <IconButton
            onClick={onCopy}
            aria-label={
              copyLabel ??
              // TRANSLATORS: Provided to accessibility tools such as screenreaders to describe a button
              // TRANSLATORS: which copies the value to the clipboard.
              messages.pgettext('accessibility', 'Copy to clipboard')
            }>
            <IconButton.Icon icon={'copy'} />
          </IconButton>
        )}
      </Flex>
    </StyledLabelContainer>
  );
}
