import { useCallback } from 'react';
import styled from 'styled-components';
import { colors } from '../../config.json';
import { messages } from '../../shared/gettext';
import log from '../../shared/logging';
import { useScheduler } from '../../shared/scheduler';
import { useBoolean } from '../lib/utilityHooks';
import ImageView from './ImageView';

const COPIED_ICON_DURATION = 2000;

interface IProps {
  value: string;
  displayValue?: string;
  obscureValue?: boolean;
  message?: string;
  className?: string;
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

const StyledButton = styled.button({
  cursor: 'default',
  padding: 0,
  marginLeft: '20px',
  backgroundColor: 'transparent',
  border: 'none',
});

const StyledCopyButton = styled(StyledButton)({
  width: '24px',
});

export default function ClipboardLabel(props: IProps) {
  const [obscured, , , toggleObscured] = useBoolean(props.obscureValue ?? true);
  const [justCopied, setJustCopied, resetJustCopied] = useBoolean(false);

  const copiedScheduler = useScheduler();

  const onCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(props.value);
      copiedScheduler.schedule(resetJustCopied, COPIED_ICON_DURATION);
      setJustCopied();
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to copy to clipboard: ${error.message}`);
    }
  }, [props.value, copiedScheduler, setJustCopied, resetJustCopied]);

  const value = props.displayValue ?? props.value;
  return (
    <StyledLabelContainer>
      <StyledLabel className={props.className} aria-hidden={obscured}>
        {obscured ? '●●●● ●●●● ●●●● ●●●●' : value}
      </StyledLabel>
      {props.obscureValue !== false && (
        <StyledButton
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
          }>
          <ImageView
            source={obscured ? 'icon-unobscure' : 'icon-obscure'}
            tintColor={colors.white}
            tintHoverColor={colors.white80}
            width={24}
          />
        </StyledButton>
      )}
      <StyledCopyButton
        onClick={onCopy}
        aria-label={
          // TRANSLATORS: Provided to accessibility tools such as screenreaders to describe a button
          // TRANSLATORS: which copies the account number to the clipboard.
          messages.pgettext('accessibility', 'Copy account number')
        }>
        <ImageView
          source={justCopied ? 'icon-tick' : 'icon-copy'}
          tintColor={justCopied ? colors.green : colors.white}
          tintHoverColor={justCopied ? colors.green : colors.white80}
          width={justCopied ? 22 : 24}
        />
      </StyledCopyButton>
    </StyledLabelContainer>
  );
}
