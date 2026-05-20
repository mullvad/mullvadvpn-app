# Location selector

## Examples

```tsx
const StyledTextFieldContent = styled(FlexColumn)`
  flex: 1;
`;

export function LocationSelectorExample() {
  const [selectedItem, setSelectedItem] = React.useState<string | undefined>('0');
  const [placeholderData, setPlaceholderData] = React.useState(['0']);
  const [isolatedItem, setIsolatedItem] = React.useState<string | undefined>(undefined);
  const [expanded, setExpanded] = React.useState(true);

  const addPlaceholderData = React.useCallback(() => {
    setPlaceholderData((prev) => [...prev, String(prev.length)]);
  }, []);

  const removePlaceholderData = React.useCallback(() => {
    setPlaceholderData((prev) => prev.slice(0, -1));
  }, []);

  const toggleExpanded = React.useCallback(() => {
    setExpanded((prev) => !prev);
  }, []);

  const handleOnItemInputChange = React.useCallback((item: string, value: string) => {
    if (value) {
      setIsolatedItem(item);
    } else {
      setIsolatedItem(undefined);
    }
  }, []);

  return (
    <FlexColumn gap="small">
      <FlexRow gap="medium">
        <Button onClick={addPlaceholderData}>
          <Button.Text>Add row</Button.Text>
        </Button>
        <Button onClick={removePlaceholderData}>
          <Button.Text>Remove row</Button.Text>
        </Button>
        <Button onClick={toggleExpanded}>
          <Button.Text>{expanded ? 'Collapse' : 'Expand'}</Button.Text>
        </Button>
      </FlexRow>

      <LocationSelector
        selectedItem={selectedItem}
        onSelectedItemChange={setSelectedItem}
        onItemInputChange={handleOnItemInputChange}
        expanded={expanded && isolatedItem === undefined}
        variant={placeholderData.length > 1 ? 'secondary' : 'primary'}>
        <LocationSelector.Row position="top">
          <LocationSelector.Row.Icon icon="device" />
          <LocationSelector.Row.Label>Your device</LocationSelector.Row.Label>
        </LocationSelector.Row>
        <LocationSelector.Items>
          {(isolatedItem ? [isolatedItem] : placeholderData).map((item, index) => {
            return (
              <LocationSelector.Items.Item
                key={item}
                id={item}
                type={index === 0 ? 'entry' : 'exit'}>
                <LocationSelector.Items.Item.TextField>
                  <StyledTextFieldContent gap="tiny">
                    <LocationSelector.Items.Item.TextField.Input placeholder={'Location ' + item} />
                  </StyledTextFieldContent>
                </LocationSelector.Items.Item.TextField>
              </LocationSelector.Items.Item>
            );
          })}
        </LocationSelector.Items>
        <LocationSelector.Row position="bottom">
          <LocationSelector.Row.Icon icon="device" />
          <LocationSelector.Row.Label>Internet</LocationSelector.Row.Label>
        </LocationSelector.Row>
      </LocationSelector>
    </FlexColumn>
  );
}
```
