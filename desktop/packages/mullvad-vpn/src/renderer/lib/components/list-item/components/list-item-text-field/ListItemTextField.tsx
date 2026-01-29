import { TextField, TextFieldProps } from '../../../text-field';
import { ListItemTextFieldInput } from './components';

export type ListItemTextFieldProps = TextFieldProps & {
  formRef?: React.RefObject<HTMLFormElement | null>;
  onSubmit?: (event: React.FormEvent) => Promise<void>;
};

function ListItemTextField({
  invalid,
  onSubmit,
  formRef,
  children,
  ...props
}: ListItemTextFieldProps) {
  return (
    <form ref={formRef} onSubmit={onSubmit}>
      <TextField invalid={invalid} {...props}>
        {children}
      </TextField>
    </form>
  );
}

const ListItemTextFieldNamespace = Object.assign(ListItemTextField, {
  Input: ListItemTextFieldInput,
});

export { ListItemTextFieldNamespace as ListItemTextField };
