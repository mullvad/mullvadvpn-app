# Menu option

## Example

Parameters:

- `disabled`: A boolean that controls whether the menu option is disabled or not.

Subcomponents:

- `Item`: The menu option item, usually contains the label and icon.
- `Label`: The label for the menu option.
- `Icon`: Decorative icon for the menu option.
- `Trigger`: A wrapper for the menu option that makes it interactive.

```tsx
  <MenuOption>
    <MenuOption.Item>
      <MenuOption.Item.Label>Options</MenuOption.Item.Label>
    </MenuOption.Item>
  </MenuOption>
  <MenuOption>
    <MenuOption.Trigger>
      <MenuOption.Item>
        <MenuOption.Item.Icon icon="search" />
        <MenuOption.Item.Label>Option 1</MenuOption.Item.Label>
      </MenuOption.Item>
    </MenuOption.Trigger>
  </MenuOption>
```
