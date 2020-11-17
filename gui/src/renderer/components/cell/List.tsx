import React, { useCallback } from 'react';
import styled from 'styled-components';
import { colors } from '../../../config.json';
import { messages } from '../../../shared/gettext';
import { AriaDescribed, AriaDescription, AriaDescriptionGroup } from '../AriaGroup';
import ImageView from '../ImageView';
import * as Cell from '.';

export interface ICellListItem<T> {
  label: string;
  value: T;
}

interface ICellListProps<T> {
  title?: string;
  items: Array<ICellListItem<T>>;
  onSelect?: (value: T) => void;
  onRemove?: (value: T) => void;
  className?: string;
  paddingLeft?: number;
}

export default function CellList<T>(props: ICellListProps<T>) {
  const paddingLeft = props.paddingLeft ?? 32;

  return (
    <Cell.Section role="listbox" className={props.className}>
      {props.title && <Cell.SectionTitle as="label">{props.title}</Cell.SectionTitle>}
      {props.items.map((item, i) => {
        return (
          <CellListItem
            key={`${i}-${item.value}`}
            value={item.value}
            onSelect={props.onSelect}
            onRemove={props.onRemove}
            paddingLeft={paddingLeft}>
            {item.label}
          </CellListItem>
        );
      })}
    </Cell.Section>
  );
}

const StyledContainer = styled(Cell.Container)({
  display: 'flex',
  marginBottom: '1px',
  backgroundColor: colors.blue40,
});

const StyledButton = styled.button({
  display: 'flex',
  alignItems: 'center',
  flex: 1,
  border: 'none',
  background: 'transparent',
  padding: 0,
  margin: 0,
});

const StyledLabel = styled(Cell.Label)({}, (props: { paddingLeft: number }) => ({
  fontFamily: 'Open Sans',
  fontWeight: 'normal',
  fontSize: '16px',
  paddingLeft: props.paddingLeft + 'px',
  whiteSpace: 'pre-wrap',
  overflowWrap: 'break-word',
  width: '171px',
  marginRight: '25px',
}));

const StyledRemoveButton = styled.button({
  background: 'transparent',
  border: 'none',
  padding: 0,
});

const StyledRemoveIcon = styled(ImageView)({
  [StyledRemoveButton + ':hover &']: {
    backgroundColor: colors.white80,
  },
});

interface ICellListItemProps<T> {
  value: T;
  onSelect?: (application: T) => void;
  onRemove?: (application: T) => void;
  paddingLeft: number;
  children: string;
}

function CellListItem<T>(props: ICellListItemProps<T>) {
  const onSelect = useCallback(() => props.onSelect?.(props.value), [props.onSelect, props.value]);
  const onRemove = useCallback(() => props.onRemove?.(props.value), [props.onRemove, props.value]);

  return (
    <AriaDescriptionGroup>
      <StyledContainer>
        <StyledButton
          onClick={props.onSelect ? onSelect : undefined}
          as={props.onSelect ? 'button' : 'span'}>
          <AriaDescription>
            <StyledLabel paddingLeft={props.paddingLeft}>{props.children}</StyledLabel>
          </AriaDescription>
        </StyledButton>
        {props.onRemove && (
          <AriaDescribed>
            <StyledRemoveButton
              onClick={onRemove}
              aria-label={messages.pgettext('accessibility', 'Remove item')}>
              <StyledRemoveIcon
                source="icon-close"
                width={22}
                height={22}
                tintColor={colors.white60}
              />
            </StyledRemoveButton>
          </AriaDescribed>
        )}
      </StyledContainer>
    </AriaDescriptionGroup>
  );
}
