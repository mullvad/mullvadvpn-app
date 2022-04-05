import React, { useCallback, useEffect, useRef } from 'react';

import { useCombinedRefs } from '../lib/utilityHooks';

interface IFormattableTextInputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  allowedCharacters: string;
  separator: string;
  uppercaseOnly?: boolean;
  maxLength?: number;
  groupLength: number;
  addTrailingSeparator?: boolean;
  handleChange: (value: string) => void;
}

function FormattableTextInput(
  props: IFormattableTextInputProps,
  forwardedRef: React.Ref<HTMLInputElement>,
) {
  const {
    addTrailingSeparator,
    allowedCharacters,
    groupLength,
    handleChange,
    maxLength,
    separator,
    uppercaseOnly,
    value,
    ...otherProps
  } = props;

  const ref = useRef() as React.RefObject<HTMLInputElement>;
  const combinedRef = useCombinedRefs(ref, forwardedRef);

  const unformat = useCallback(
    (value: string) => {
      const correctCaseValue = uppercaseOnly ? value.toUpperCase() : value;
      return correctCaseValue.match(new RegExp(allowedCharacters, 'g'))?.join('') ?? '';
    },
    [uppercaseOnly, allowedCharacters],
  );

  const format = useCallback(
    (value: string, addTrailingSeparator?: boolean) => {
      let formatted = value.match(new RegExp(`.{1,${groupLength}}`, 'g'))?.join(separator) ?? '';

      if (
        addTrailingSeparator &&
        value.length > 0 &&
        value.length % groupLength === 0 &&
        (!maxLength || maxLength > value.length)
      ) {
        formatted += separator;
      }

      return formatted;
    },
    [groupLength, separator, maxLength],
  );

  const onBeforeInput = useCallback(
    (event: Event) => {
      const { inputType, data, target } = event as InputEvent;

      if (ref.current) {
        const inputElement = target as HTMLInputElement;
        const oldValue = inputElement.value;

        const selectionStart = inputElement.selectionStart ?? oldValue.length;
        const selectionEnd = inputElement.selectionEnd ?? selectionStart;
        const emptySelection = selectionStart === selectionEnd;
        const beforeSelection = unformat(oldValue.slice(0, selectionStart));
        const afterSelection = unformat(oldValue.slice(selectionEnd));

        let unformattedData = unformat(data ?? '');
        // Only allow adding data that fits into the max length.
        if (maxLength) {
          const charactersLeft = maxLength - beforeSelection.length - afterSelection.length;
          unformattedData = unformattedData.slice(0, charactersLeft);
        }

        let newValue: string;
        let caretPosition: number;
        if (inputType === 'deleteContentBackward' && emptySelection && beforeSelection.length > 0) {
          // This is triggered when pressing backspace without a selection
          newValue = beforeSelection.slice(0, -1) + afterSelection;
          caretPosition = format(beforeSelection + unformattedData, false).length - 1;
        } else if (inputType === 'deleteContentForward' && emptySelection) {
          // This is triggered when pressing delete without a selection
          newValue = beforeSelection + afterSelection.slice(1);
          caretPosition = format(beforeSelection + unformattedData, true).length;
        } else {
          newValue = beforeSelection + unformattedData + afterSelection;
          caretPosition = format(beforeSelection + unformattedData, true).length;
        }

        const formattedValue = format(newValue, addTrailingSeparator);
        caretPosition = Math.min(caretPosition, formattedValue.length);

        // The new value can't be set before the browser has changed the content of the input
        // element since that would result in the change being made twice. Another alternative would
        // be to call `event.preventDefault()` but that prevents other side effects such as the
        // scrolling of the input content when overflowing.
        ref.current.addEventListener(
          'input',
          () => {
            inputElement.value = formattedValue;
            inputElement.selectionStart = inputElement.selectionEnd = caretPosition;
            handleChange(newValue);
          },
          { once: true },
        );
      }
    },
    [unformat, format, handleChange, addTrailingSeparator],
  );

  // React doesn't fully support onBeforeInput currently and it's therefore set here.
  useEffect(() => {
    ref.current?.addEventListener('beforeinput', onBeforeInput);
    return () => ref.current?.removeEventListener('beforeinput', onBeforeInput);
  }, [onBeforeInput]);

  // Use value provided in props if it differs from current input value.
  useEffect(() => {
    if (typeof value === 'string' && ref.current && unformat(ref.current.value) !== value) {
      ref.current.value = format(value, addTrailingSeparator);
    }
  }, [format, value, addTrailingSeparator]);

  return <input ref={combinedRef} type="text" {...otherProps} />;
}

export default React.memo(React.forwardRef(FormattableTextInput));
