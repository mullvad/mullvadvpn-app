import * as uuid from 'uuid';

export function createInputAriaAttributes() {
  const inputId = uuid.v4();
  const labelId = `${inputId}-label`;
  const descriptionId = `${inputId}-description`;

  return {
    label: {
      id: labelId,
      htmlFor: inputId,
    },
    input: {
      id: inputId,
      'aria-labelledby': labelId,
      'aria-describedby': descriptionId,
    },
    description: {
      id: descriptionId,
    },
  };
}
