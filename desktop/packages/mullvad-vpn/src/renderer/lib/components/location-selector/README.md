# Location selector

## Examples

```tsx

  const [selectedItem, setSelectedItem] = React.useState<string | undefined>('0');
  const [isolatedItem, setIsolatedItem] = React.useState<string | undefined>(undefined);
  const [placeholderData, setPlaceholderData] = React.useState<Record<string, string | undefined>>({
    '0': undefined,
  });

  const handleSelectedItemChange = React.useCallback((itemId: string) => {
    setSelectedItem(itemId);
  }, []);

  const addPlaceholderData = React.useCallback(() => {
    setPlaceholderData((prev) => {
      const newKey = Object.keys(prev).length;
      return { ...prev, [String(newKey)]: undefined };
    });
  }, []);

  const removePlaceholderData = React.useCallback(() => {
    setPlaceholderData((prev) => {
      const newData = { ...prev };
      const keys = Object.keys(newData);
      if (keys.length === 0) {
        return newData;
      }

      const { [keys[keys.length - 1]]: _, ...rest } = newData;
      return rest;
    });
  }, []);

  const handleOnValueChange = React.useCallback(
    (item: string, value: string) => {
      if (value && !isolatedItem) {
        setIsolatedItem(item);
      } else if (!value && isolatedItem === item) {
        setIsolatedItem(undefined);
      }
      setPlaceholderData((prev) => ({ ...prev, [item]: value }));
    },
    [isolatedItem],
  );

  return (
    <FlexColumn gap="small">
      <FlexRow gap="medium">
        <Button onClick={addPlaceholderData}>
          <Button.Text>Add row</Button.Text>
        </Button>
        <Button onClick={removePlaceholderData}>
          <Button.Text>Remove row</Button.Text>
        </Button>
      </FlexRow>

      <LocationSelector
        selectedItem={selectedItem}
        onSelectedItemChange={handleSelectedItemChange}
        expanded={isolatedItem === undefined}
        variant={Object.keys(placeholderData).length > 1 ? 'secondary' : 'primary'}>
        <LocationSelector.Row position="top">
          <LocationSelector.Row.Icon icon="device" />
          <LocationSelector.Row.Label>Your device</LocationSelector.Row.Label>
        </LocationSelector.Row>
        <LocationSelector.Items>
          {(isolatedItem ? [isolatedItem] : Object.keys(placeholderData)).map((item, index) => {
            return (
              <LocationSelector.Items.Item
                key={item}
                id={item}
                type={index === 0 ? 'entry' : 'exit'}>
                <LocationSelector.Items.Item.TextField
                  value={placeholderData[item]}
                  onValueChange={handleOnValueChange}>
                  <LocationSelector.Items.Item.TextField.Input placeholder={'Location ' + item} />
                  <LocationSelector.Items.Item.TextField.ClearButton />
                </LocationSelector.Items.Item.TextField>
                <LocationSelector.Items.Item.TrailingButton visible={isolatedItem === undefined}>
                  <LocationSelector.Items.Item.TrailingButton.Icon icon="filter" />
                </LocationSelector.Items.Item.TrailingButton>
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
