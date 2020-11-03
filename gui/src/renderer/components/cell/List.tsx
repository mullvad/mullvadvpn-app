import React, { useCallback } from 'react';
import styled from 'styled-components';
import { colors } from '../../../config.json';
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
    <Cell.CellButton onClick={props.onSelect ? onSelect : undefined}>
      <StyledLabel paddingLeft={props.paddingLeft}>{props.children}</StyledLabel>
      {props.onRemove && (
        <ImageView
          source="icon-close"
          width={22}
          height={22}
          onClick={onRemove}
          tintColor={colors.white60}
          tintHoverColor={colors.white80}
        />
      )}
    </Cell.CellButton>
  );
}
