import { useCallback, useEffect, useRef, useState } from 'react';
import styled from 'styled-components';

import { Scheduler } from '../../shared/scheduler';
import Accordion from './Accordion';

export const stringValueAsKey = (value: string): string => value;

const StyledListItem = styled.div({
  display: 'flex',
  flex: 1,
  flexDirection: 'column',
});

interface ListProps<T> {
  items: Array<T>;
  getKey: (data: T) => string;
  children: (data: T) => React.ReactNode;
  skipAddTransition?: boolean;
  skipInitialAddTransition?: boolean;
  skipRemoveTransition?: boolean;
}

export interface RowData<T> {
  key: string;
  data: T;
}

export interface RowDisplayData<T> extends RowData<T> {
  removing: boolean;
}

export default function List<T>(props: ListProps<T>) {
  const [displayItems, setDisplayItems] = useState(() =>
    convertToRowDisplayData(props.items, props.getKey),
  );
  // Skip add transition on first render when initial items are added.
  const skipAddTransition = useRef(props.skipInitialAddTransition ?? false);

  const removeFallbackSchedulers = useRef<Record<string, Scheduler>>({});

  useEffect(() => {
    setDisplayItems((prevItems) => {
      if (props.skipRemoveTransition) {
        return convertToRowDisplayData(props.items, props.getKey);
      } else {
        const nextItems = convertToRowData(props.items, props.getKey);
        return calculateItemList(prevItems, nextItems);
      }
    });
  }, [props.items, props.getKey]);

  useEffect(() => {
    // Set to animate accordion for added items after first render unless
    // props.skipAddTransition === true.
    skipAddTransition.current = props.skipAddTransition ?? false;
  }, []);

  const onRemoved = useCallback((key: string) => {
    removeFallbackSchedulers.current[key].cancel();
    delete removeFallbackSchedulers.current[key];

    setDisplayItems((items) => items.filter((item) => item.key !== key));
  }, []);

  useEffect(() => {
    // Add scheduled item removal if `onTransitionEnd` doesn't trigger for some reason.
    displayItems
      .filter((item) => item.removing && removeFallbackSchedulers.current[item.key] === undefined)
      .forEach((item) => {
        const scheduler = new Scheduler();
        scheduler.schedule(() => onRemoved(item.key), 400);
        removeFallbackSchedulers.current[item.key] = scheduler;
      });
  }, [displayItems]);

  useEffect(
    () => () => {
      // Cancel all schedulers on unmount
      Object.values(removeFallbackSchedulers.current).forEach((scheduler) => scheduler.cancel());
    },
    [],
  );

  return (
    <>
      {displayItems.map((displayItem) => (
        <ListItem
          key={displayItem.key}
          data={displayItem}
          onRemoved={onRemoved}
          render={props.children}
          skipAddTransition={skipAddTransition.current}
        />
      ))}
    </>
  );
}

interface ListItemProps<T> {
  data: RowDisplayData<T>;
  onRemoved: (key: string) => void;
  render: (data: T) => React.ReactNode;
  skipAddTransition: boolean;
}

function ListItem<T>(props: ListItemProps<T>) {
  // If skipAddTransition is true then the item is expanded from the beginning.
  const [expanded, setExpanded] = useState(props.skipAddTransition);

  const onTransitionEnd = useCallback(() => {
    if (props.data.removing) {
      props.onRemoved(props.data.key);
    }
  }, [props.onRemoved, props.data.key, props.data.removing]);

  // Expands after initial render and collapses when item is set to being removed.
  useEffect(() => setExpanded(!props.data.removing), [props.data.removing]);

  return (
    <Accordion expanded={expanded} onTransitionEnd={onTransitionEnd}>
      <StyledListItem>{props.render(props.data.data)}</StyledListItem>
    </Accordion>
  );
}

function convertToRowData<T>(items: Array<T>, getKey: (data: T) => string): Array<RowData<T>> {
  return items.map((item) => ({ key: getKey(item), data: item }));
}

function convertToRowDisplayData<T>(
  items: Array<T>,
  getKey: (data: T) => string,
  removing = false,
): Array<RowDisplayData<T>> {
  return convertToRowData(items, getKey).map((item) => ({ ...item, removing }));
}

export function calculateItemList<T>(
  prevItemsList: Array<RowDisplayData<T>>,
  nextItemsList: Array<RowData<T>>,
): Array<RowDisplayData<T>> {
  const prevItems = [...prevItemsList];
  const nextItems = [...nextItemsList];

  if (
    prevItems.length !== nextItems.length ||
    !prevItems.every((prevItem, i) => prevItem.key === nextItems[i].key)
  ) {
    // If the nextItems contains changes from prevItems we want to calculate the next state.
    const combinedItems: Array<RowDisplayData<T>> = [];

    while (prevItems.length > 0 || nextItems.length > 0) {
      const prevItem = prevItems[0];
      const nextItem = nextItems[0];

      // Either prevItem or nextItem must have a value since at least one of the lists isn't
      // empty.
      if (prevItem?.key === nextItem?.key) {
        combinedItems.push({ ...prevItem, removing: false });
        prevItems.shift();
        nextItems.shift();
      } else if (
        prevItem === undefined ||
        nextItems.find((item) => item.key === prevItem.key) !== undefined
      ) {
        // An item has been added if there are no more previous items or if the current prevItem
        // exists later in nextItems.
        combinedItems.push({ ...nextItem, removing: false });
        nextItems.shift();
      } else {
        combinedItems.push({ ...prevItem, removing: true });
        prevItems.shift();
      }
    }

    return combinedItems;
  } else {
    return prevItemsList;
  }
}
