# List Item

## Examples

### List item with actions and footer

```tsx
export function ListItemWithActionsAndFooter() {
  return (
    <ListItem>
      <ListItem.Item>
        <ListItem.Item.Label>Title</ListItem.Item.Label>
        <ListItem.Item.ActionGroup>
          <IconButton>
            <IconButton.Icon icon="add-circle" />
          </IconButton>
          <IconButton>
            <IconButton.Icon icon="remove-circle" />
          </IconButton>
        </ListItem.Item.ActionGroup>
      </ListItem.Item>
      <ListItem.Footer>
        <ListItem.Footer.Text>Description</ListItem.Footer.Text>
      </ListItem.Footer>
    </ListItem>
  );
}
```

### Clickable list Item

```tsx
export function ClickableListItem() {
  return (
    <ListItem>
      <ListItem.Trigger onClick={() => console.log('list item clicked')}>
        <ListItem.Item>
          <ListItem.Item.Label>Title</ListItem.Item.Label>
        </ListItem.Item>
      </ListItem.Trigger>
    </ListItem>
  );
}
```

### List item with trailing action

```tsx
export function ListItemWithTrailingAction() {
  return (
    <ListItem>
      <ListItem.Item>
        <ListItem.Item.Label>Title</ListItem.Item.Label>
      </ListItem.Item>
      <ListItem.TrailingActions>
        <ListItem.Trigger onClick={() => console.log('trailing action clicked')}>
          <ListItem.TrailingActions.Action>
            <ListItem.TrailingActions.Action.Icon icon="search"></ListItem.TrailingActions.Action.Icon>
          </ListItem.TrailingActions.Action>
        </ListItem.Trigger>
      </ListItem.TrailingActions>
    </ListItem>
  );
}
```
