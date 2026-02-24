# Menu

Uses the `Menu option` atom.

Parameters:

- `open`: A boolean that controls whether the menu is open or closed, usually a state variable.
- `onOpenChange`: A callback function that is called when the open state of the menu changes.
- `triggerRef`: A reference to the element that triggers the menu.

Subcomponents:

- `Popup`: The popover that contains the menu options.
- `Title`: A title for the menu, usually displayed at the top of the popup.
- `Divider`: A divider that separates menu options.
- `Option`: A menu option, see menu option components for more details.

## Example

```tsx
<Menu open={menuOpen} onOpenChange={setMenuOpen} triggerRef={triggerRef}>
  <Menu.Popup>
    <Menu.Title>Options</Menu.Title>
    <Menu.Option>
      <Menu.Option.Trigger>
        <Menu.Option.Item>
          <Menu.Option.ItemIcon icon="search" />
          <Menu.Option.ItemLabel>Option 1</Menu.Option.ItemLabel>
        </Menu.Option.Item>
      </Menu.Option.Trigger>
    </Menu.Option>
    <Menu.Divider />
    <Menu.Option.Item>
      <Menu.Option.ItemIcon icon="search" />
      <Menu.Option.ItemLabel>Option 2</Menu.Option.ItemLabel>
    </Menu.Option.Item>
  </Menu.Popup>
</Menu>
```
