import { useCallback } from 'react';
import styled from 'styled-components';

import { messages } from '../../shared/gettext';
import log from '../../shared/logging';
import { useScheduler } from '../../shared/scheduler';
import { Flex, Icon, IconButton } from '../lib/components';
import { Colors, Spacings } from '../lib/foundations';
import { useBoolean } from '../lib/utility-hooks';

const COPIED_ICON_DURATION = 2000;

interface IProps extends React.HTMLAttributes<HTMLElement> {
  value: string;
  displayValue?: string;
  obscureValue?: boolean;
  message?: string;
}

const StyledLabelContainer = styled.div({
  display: 'flex',
  flex: 1,
  height: '19px',
  alignItems: 'center',
});

const StyledLabel = styled.span({
  flex: 1,
});

export default function ClipboardLabel(props: IProps) {
  const { value, obscureValue, displayValue, message, ...otherProps } = props;

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
      <Flex $gap={Spacings.spacing5}>
        {obscureValue !== false && (
          <IconButton
            icon={obscured ? 'show' : 'hide'}
            onClick={toggleObscured}
            aria-label={
              obscured
                ? // This line is here to prevent the following one to be moved up here by prettier
                  // TRANSLATORS: Provided to accessibility tools such as screenreaders to describe
                  // TRANSLATORS: the button which unobscures the account number.
                  messages.pgettext('accessibility', 'Show account number')
                : // This line is here to prevent the following one to be moved up here by prettier
                  // TRANSLATORS: Provided to accessibility tools such as screenreaders to describe
                  // TRANSLATORS: the button which obscures the account number.
                  messages.pgettext('accessibility', 'Hide account number')
            }
          />
        )}
        {justCopied ? (
          <Icon icon="checkmark" color={Colors.green}></Icon>
        ) : (
          <IconButton
            icon={'copy'}
            onClick={onCopy}
            aria-label={
              // TRANSLATORS: Provided to accessibility tools such as screenreaders to describe a button
              // TRANSLATORS: which copies the account number to the clipboard.
              messages.pgettext('accessibility', 'Copy account number')
            }
          />
        )}
      </Flex>
    </StyledLabelContainer>
  );
}
