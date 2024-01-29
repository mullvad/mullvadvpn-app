import { useCallback, useId, useState } from 'react';
import { styled } from 'styled-components';

import { colors } from '../../../config.json';
import { AriaInput, AriaInputGroup, AriaLabel } from '../AriaGroup';
import { smallNormalText } from '../common-styles';
import { SettingsSelectItem } from './SettingsSelect';

const StyledRadioGroup = styled.div({
  display: 'flex',
});

interface SettingsSelectProps<T extends string> {
  defaultValue?: T;
  items: Array<SettingsSelectItem<T>>;
  onUpdate: (value: T) => void;
}

export function SettingsRadioGroup<T extends string>(props: SettingsSelectProps<T>) {
  const [value, setValue] = useState<T>(props.defaultValue ?? props.items[0]?.value ?? '');
  const key = useId();

  const onSelect = useCallback((value: T) => {
    setValue(value);
    props.onUpdate(value);
  }, []);

  return (
    <StyledRadioGroup>
      {props.items.map((item) => (
        <RadioButton
          key={item.value}
          group={key}
          item={item}
          selected={item.value === value}
          onSelect={onSelect}
        />
      ))}
    </StyledRadioGroup>
  );
}

const StyledRadioButton = styled.input.attrs({ type: 'radio' })({
  position: 'relative',
  margin: 0,
  appearance: 'none',
  backgroundColor: 'transparent',
  width: '12px',
  height: '12px',

  '&&::before': {
    position: 'absolute',
    content: '""',
    width: '12px',
    height: '12px',
    borderRadius: '50%',
    backgroundColor: 'transparent',
    border: `1px ${colors.white} solid`,
    top: 0,
    left: 0,
  },

  '&&:checked::after': {
    position: 'absolute',
    content: '""',
    width: '8px',
    height: '8px',
    borderRadius: '50%',
    backgroundColor: colors.white,
    top: '3px',
    left: '3px',
  },
});

const StyledRadioButtonContainer = styled.div({
  display: 'flex',
  alignItems: 'center',
  flexWrap: 'nowrap',
  marginLeft: '16px',
});

const StyledRadioButtonLabel = styled.label(smallNormalText, {
  color: colors.white,
  marginLeft: '8px',
});

interface RadioButtonProps<T extends string> {
  group: string;
  item: SettingsSelectItem<T>;
  selected: boolean;
  onSelect: (value: T) => void;
}

function RadioButton<T extends string>(props: RadioButtonProps<T>) {
  const onChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      props.onSelect(event.target.value as T);
    },
    [props.onSelect],
  );

  return (
    <StyledRadioButtonContainer>
      <AriaInputGroup>
        <AriaInput>
          <StyledRadioButton
            name={props.group}
            value={props.item.value}
            onChange={onChange}
            checked={props.selected}
          />
        </AriaInput>
        <AriaLabel>
          <StyledRadioButtonLabel>{props.item.label}</StyledRadioButtonLabel>
        </AriaLabel>
      </AriaInputGroup>
    </StyledRadioButtonContainer>
  );
}
