import { TextField, type TextFieldProps } from '../../../../../text-field';
import { ListItemItemTextFieldInput } from './components';

export type ListItemItemTextFieldProps = TextFieldProps & {
  formRef?: React.RefObject<HTMLFormElement | null>;
  onSubmit?: (event: React.FormEvent) => Promise<void>;
};

function ListItemItemTextField({
  invalid,
  onSubmit,
  formRef,
  children,
  ...props
}: ListItemItemTextFieldProps) {
  return (
    <form ref={formRef} onSubmit={onSubmit}>
      <TextField invalid={invalid} {...props}>
        <TextField.InputGroup>{children}</TextField.InputGroup>
      </TextField>
    </form>
  );
}

const ListItemItemTextFieldNamespace = Object.assign(ListItemItemTextField, {
  Input: ListItemItemTextFieldInput,
});

export { ListItemItemTextFieldNamespace as ListItemItemTextField };
