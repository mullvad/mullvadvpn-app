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
      <TextField.Icon icon="search" />
      <TextField.Input placeholder="Enter text" inputMode="text" maxLength={100} />
    </TextField>
  );
}
```
