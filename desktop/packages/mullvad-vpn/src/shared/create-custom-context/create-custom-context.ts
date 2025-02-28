import { createContext } from 'react';

import type { CustomContext, CustomContextReact, CustomContextValues } from './types';

const createCustomContext = <Values extends CustomContextValues>(
  values?: Values,
): CustomContextReact<Values> => {
  const context = createContext<CustomContext<Values>>({
    values: values as Values,
    setValues: () => {},
  });

  return context;
};

export default createCustomContext;
