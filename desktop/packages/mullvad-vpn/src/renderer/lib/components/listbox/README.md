# Listbox

Header and options use `ListItem` component.

## Examples

```tsx
export function Accordion() {
  const [value, setValue] = React.useState('value 1');

  return (
    <Listbox value={value} onValueChange={setValue}>
      <Listbox.Header>
        <Listbox.Header.Item>
          <Listbox.Header.Item.Label>Values</Listbox.Header.Item.Label>
        </Listbox.Header.Item>
      </Listbox.Header>
      <Listbox.Options>
        <Listbox.Options.Option value={'value 1'}>
          <Listbox.Options.Option.Trigger>
            <Listbox.Options.Option.Item>
              <Listbox.Options.Option.Item.Label>Option 1</Listbox.Options.Option.Item.Label>
            </Listbox.Options.Option.Item>
          </Listbox.Options.Option.Trigger>
        </Listbox.Options.Option>
        <Listbox.Options.Option value={'value 2'}>
          <Listbox.Options.Option.Trigger>
            <Listbox.Options.Option.Item>
              <Listbox.Options.Option.Item.Label>Option 2</Listbox.Options.Option.Item.Label>
            </Listbox.Options.Option.Item>
          </Listbox.Options.Option.Trigger>
        </Listbox.Options.Option>
      </Listbox.Options>
    </Listbox>
  );
}
```
