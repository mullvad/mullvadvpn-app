import { useCallback } from 'react';
import styled from 'styled-components';

import { Icon } from '../../../../../lib/components';
import { colors } from '../../../../../lib/foundations';
import * as Cell from '../../../../cell';
import { normalText } from '../../../../common-styles';

interface IStyledRowTitleProps {
  $bold?: boolean;
}

const StyledCheckbox = styled.div({
  width: '24px',
  height: '24px',
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  backgroundColor: colors.white,
  borderRadius: '4px',
});

const StyledRow = styled(Cell.Row)({
  backgroundColor: colors.blue40,
  '&&:hover': {
    backgroundColor: colors.blue80,
  },
});

const StyledRowTitle = styled.label<IStyledRowTitleProps>(normalText, (props) => ({
  fontWeight: props.$bold ? 600 : 400,
  color: colors.white,
  marginLeft: '22px',
}));

interface ICheckboxRowProps extends IStyledRowTitleProps {
  label: string;
  checked: boolean;
  onChange: (provider: string) => void;
}

export function CheckboxRow(props: ICheckboxRowProps) {
  const { onChange } = props;

  const onToggle = useCallback(() => onChange(props.label), [onChange, props.label]);

  return (
    <StyledRow onClick={onToggle}>
      <StyledCheckbox role="checkbox" aria-label={props.label} aria-checked={props.checked}>
        {props.checked && <Icon icon="checkmark" color="green" />}
      </StyledCheckbox>
      <StyledRowTitle aria-hidden $bold={props.$bold}>
        {props.label}
      </StyledRowTitle>
    </StyledRow>
  );
}
