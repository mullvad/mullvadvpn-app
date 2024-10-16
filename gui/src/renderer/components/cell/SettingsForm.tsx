import React, { useCallback, useContext, useEffect, useId, useMemo, useState } from 'react';

import { useEffectEvent } from '../../lib/utility-hooks';

interface SettingsFormContext {
  formSubmittable: boolean;
  reportInputSubmittable: (key: string, submittable: boolean) => void;
  removeInput: (key: string) => void;
}

// Keep track of all submittable and non submittable inputs in a form to enable e.g. buttons to
// become enabled/disabled based on input states.
const settingsFormContext = React.createContext<SettingsFormContext | undefined>(undefined);

function useSettingsFormContext() {
  return useContext(settingsFormContext);
}

// Hook that returns whether or not the form is submittable for use in form container.
export function useSettingsFormSubmittable() {
  const context = useSettingsFormContext();
  return context?.formSubmittable ?? true;
}

// Hook that returns function that input can use to report if it's submittable or not.
export function useSettingsFormSubmittableReporter() {
  const context = useSettingsFormContext();

  // Each form needs an unique ID, this key is part of that ID.
  const key = useId();

  const reportInputSubmittable = useCallback(
    (submittable: boolean) => {
      context?.reportInputSubmittable(key, submittable);
    },
    [context, key],
  );

  const clearRequiredFields = useEffectEvent(() => {
    context?.removeInput(key);
  });

  // Remove from required fields if unmounted.
  useEffect(() => () => clearRequiredFields(), []);

  return reportInputSubmittable;
}

export function SettingsForm(props: React.PropsWithChildren) {
  const [inputStatuses, setInputStatuses] = useState<Record<string, boolean>>({});

  const reportInputSubmittable = useCallback((key: string, submittable: boolean) => {
    setInputStatuses((prevInputStatuses) => ({ ...prevInputStatuses, [key]: submittable }));
  }, []);

  const removeInput = useCallback((key: string) => {
    setInputStatuses((prevInputStatuses) => {
      const { [key]: _, ...inputStatuses } = prevInputStatuses;
      return inputStatuses;
    });
  }, []);

  const value = useMemo(
    () => ({
      formSubmittable: Object.values(inputStatuses).every((item) => item === true),
      reportInputSubmittable,
      removeInput,
    }),
    [inputStatuses, removeInput, reportInputSubmittable],
  );

  return (
    <settingsFormContext.Provider value={value}>{props.children}</settingsFormContext.Provider>
  );
}
