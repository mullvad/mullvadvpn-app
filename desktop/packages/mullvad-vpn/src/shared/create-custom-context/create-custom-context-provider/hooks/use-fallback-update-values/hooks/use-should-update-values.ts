import isEqual from 'lodash/isEqual';

import useGetPrevious from './use-get-previous';

const useShouldUpdateValues = <PropsValues extends object>(propsValues: PropsValues) => {
  const getPreviousPropsValues = useGetPrevious(propsValues);
  const previousPropsValues = getPreviousPropsValues();

  const hasPreviousValues = typeof previousPropsValues !== 'undefined';
  const valuesChanged = !isEqual(propsValues, previousPropsValues);

  const shouldUpdateValues = hasPreviousValues && valuesChanged;

  return shouldUpdateValues;
};

export default useShouldUpdateValues;
