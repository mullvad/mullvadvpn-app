# Text Field

A text input component based on the html `<input>` element. Can be used in conjunction with the
`useTextField` hook for managing state and validation.

## Example

```tsx
export function ExampleTextField() {
  const inputRef = React.useRef<HTMLInputElement | null>(null);
  const { value, handleOnValueChange, invalid } = useTextField({
    inputRef,
    defaultValue: '',
    validate: (val) => val.length < 5,
  });

  return (
    <TextField value={value} onValueChange={handleOnValueChange} invalid={invalid}>
      <TextField.Label>Some text</TextField>
      <TextField.InputGroup>
        <TextField.InputGroup.Icon icon="search" />
        <TextField.InputGroup.Input placeholder="Enter text" inputMode="text" maxLength={100} />
        <TextField.InputGroup.IconButton icon="filter">
      </TextField.InputGroup>
      <TextField.SupportingText>Enter some text above</TextField.SupportingText>
    </TextField>
  );
}
```
