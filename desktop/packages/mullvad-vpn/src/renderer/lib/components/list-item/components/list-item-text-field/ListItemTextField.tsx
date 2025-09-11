import { TextField, TextFieldProps } from '../../../text-field';
import { ListItemTextFieldInput } from './list-item-text-field-input';

export type ListItemTextFieldProps = TextFieldProps & {
  onSubmit?: (event: React.FormEvent) => Promise<void>;
};

function ListItemTextField({ invalid, onSubmit, children, ...props }: ListItemTextFieldProps) {
  return (
    <form onSubmit={onSubmit}>
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
