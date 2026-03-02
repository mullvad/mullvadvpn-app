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
      <MenuOption.ItemLabel>Options</MenuOption.ItemLabel>
    </MenuOption.Item>
  </MenuOption>
  <MenuOption>
    <MenuOption.Trigger>
      <MenuOption.Item>
        <MenuOption.ItemIcon icon="search" />
        <MenuOption.ItemLabel>Option 1</MenuOption.ItemLabel>
      </MenuOption.Item>
    </MenuOption.Trigger>
  </MenuOption>
```
