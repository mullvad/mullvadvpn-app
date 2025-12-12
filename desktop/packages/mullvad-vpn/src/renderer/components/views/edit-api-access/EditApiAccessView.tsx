import { useCallback, useState } from 'react';
import { useParams } from 'react-router';
import { sprintf } from 'sprintf-js';

import {
  CustomProxy,
  NamedCustomProxy,
  NewAccessMethodSetting,
} from '../../../../shared/daemon-rpc-types';
import { messages } from '../../../../shared/gettext';
import { useScheduler } from '../../../../shared/scheduler';
import { useAppContext } from '../../../context';
import { useApiAccessMethodTest } from '../../../lib/api-access-methods';
import { Button } from '../../../lib/components';
import { FlexColumn } from '../../../lib/components/flex-column';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { useLastDefinedValue } from '../../../lib/utility-hooks';
import { useSelector } from '../../../redux/store';
import { AppNavigationHeader } from '../..';
import { SettingsForm } from '../../cell/SettingsForm';
import { BackAction } from '../../KeyboardNavigation';
import { SettingsNavigationScrollbars } from '../../Layout';
import { ModalAlert, ModalAlertType } from '../../Modal';
import { NavigationContainer } from '../../NavigationContainer';
import {
  NamedProxyForm,
  ProxyFormButtons,
  ProxyFormInner,
  ProxyFormNameField,
} from '../../ProxyForm';
import SettingsHeader, { HeaderSubTitle, HeaderTitle } from '../../SettingsHeader';

export function EditApiAccessView() {
  return (
    <SettingsForm>
      <AccessMethodForm></AccessMethodForm>
    </SettingsForm>
  );
}

function AccessMethodForm() {
  const { pop } = useHistory();
  const { addApiAccessMethod, updateApiAccessMethod } = useAppContext();
  const methods = useSelector((state) => state.settings.apiAccessMethods.custom);

  const [testing, testResult, testApiAccessMethod, resetTestResult] = useApiAccessMethodTest(
    false,
    500,
  );
  const saveScheduler = useScheduler();

  // Use id in url to figure out which method is to be edited. undefined means this is a new method.
  const { id } = useParams<{ id: string | undefined }>();
  const method = methods.find((method) => method.id === id);

  const [updatedMethod, setUpdatedMethod] = useState<
    NewAccessMethodSetting<CustomProxy> | undefined
  >(method);

  const save = useCallback(
    (method: NewAccessMethodSetting<CustomProxy>) => {
      if (method !== undefined) {
        resetTestResult();
        if (id === undefined) {
          void addApiAccessMethod(method);
        } else {
          void updateApiAccessMethod({ ...method, id });
        }
        pop();
      }
    },
    [resetTestResult, id, pop, addApiAccessMethod, updateApiAccessMethod],
  );

  const onSave = useCallback(
    async (newMethod: NamedCustomProxy) => {
      const enabled = id === undefined ? true : (method?.enabled ?? true);
      const updatedMethod = { ...newMethod, enabled };
      setUpdatedMethod(updatedMethod);
      if (
        updatedMethod !== undefined &&
        (await testApiAccessMethod(updatedMethod as CustomProxy))
      ) {
        // Hide the save dialog after 1.5 seconds.
        saveScheduler.schedule(() => save(updatedMethod), 1500);
      }
    },
    [id, method?.enabled, testApiAccessMethod, saveScheduler, save],
  );

  const handleDialogSave = useCallback(() => {
    if (updatedMethod !== undefined) {
      save(updatedMethod);
    }
  }, [save, updatedMethod]);

  const title = getTitle(id === undefined);
  const subtitle = getSubtitle(id === undefined);

  const customAccessMethods = useSelector((state) => state.settings.apiAccessMethods.custom);
  const onValidate = useCallback(
    (value: string) => {
      const nameUsedInOtherAccessMethod = customAccessMethods.some(
        (customAccessMethod) =>
          method?.id !== customAccessMethod.id && customAccessMethod.name === value,
      );

      return !nameUsedInOtherAccessMethod;
    },
    [customAccessMethods, method],
  );

  return (
    <View backgroundColor="darkBlue">
      <BackAction action={pop}>
        <NavigationContainer>
          <AppNavigationHeader title={title} />

          <SettingsNavigationScrollbars fillContainer>
            <View.Content>
              <FlexColumn>
                <SettingsHeader>
                  <HeaderTitle>{title}</HeaderTitle>
                  <HeaderSubTitle>{subtitle}</HeaderSubTitle>
                </SettingsHeader>

                {id !== undefined && method === undefined ? (
                  <span>Failed to open method</span>
                ) : (
                  <NamedProxyForm proxy={method} onSave={onSave} onCancel={pop}>
                    <ProxyFormNameField
                      rowProps={{
                        errorMessage: messages.pgettext(
                          'api-access-methods-view',
                          'Please select a name for the access method not already in use.',
                        ),
                      }}
                      inputProps={{ validate: onValidate }}
                    />
                    <ProxyFormInner />
                    <ProxyFormButtons />
                  </NamedProxyForm>
                )}

                <TestingDialog
                  name={updatedMethod?.name ?? ''}
                  newMethod={id === undefined}
                  testing={testing}
                  testResult={testResult}
                  cancel={resetTestResult}
                  save={handleDialogSave}
                />
              </FlexColumn>
            </View.Content>
          </SettingsNavigationScrollbars>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}

function getTitle(isNewMethod: boolean) {
  return isNewMethod
    ? messages.pgettext('api-access-methods-view', 'Add method')
    : messages.pgettext('api-access-methods-view', 'Edit method');
}

function getSubtitle(isNewMethod: boolean) {
  return isNewMethod
    ? messages.pgettext('api-access-methods-view', 'Adding a new API access method also tests it.')
    : messages.pgettext('api-access-methods-view', 'Editing an API access method also tests it.');
}

interface TestingDialogProps {
  name: string;
  newMethod: boolean;
  testing: boolean;
  testResult?: boolean;
  cancel: () => void;
  save: () => void;
}

function TestingDialog(props: TestingDialogProps) {
  let currentType: ModalAlertType | undefined;
  if (props.testing) {
    currentType = ModalAlertType.loading;
  } else if (props.testResult) {
    currentType = ModalAlertType.success;
  } else if (props.testResult === false) {
    currentType = ModalAlertType.failure;
  }

  const type = useLastDefinedValue(currentType);
  const displayType = type ?? ModalAlertType.failure;

  return (
    <ModalAlert
      isOpen={!!currentType}
      type={type}
      gridButtons={getTestingDialogButtons(displayType, props.save, props.cancel)}
      close={props.cancel}
      title={getTestingDialogTitle(displayType, props.newMethod)}
      message={getTestingDialogSubTitle(displayType, props.newMethod, props.name)}
    />
  );
}

function getTestingDialogTitle(type: ModalAlertType, newMethod: boolean) {
  switch (type) {
    case ModalAlertType.success:
      return newMethod
        ? messages.pgettext('api-access-methods-view', 'API reachable, adding method…')
        : messages.pgettext('api-access-methods-view', 'API reachable, saving method…');
    case ModalAlertType.failure:
      return newMethod
        ? messages.pgettext('api-access-methods-view', 'API unreachable, add anyway?')
        : messages.pgettext('api-access-methods-view', 'API unreachable, save anyway?');
    default:
    case ModalAlertType.loading:
      return messages.pgettext('api-access-methods-view', 'Testing method...');
  }
}

function getTestingDialogSubTitle(type: ModalAlertType, newMethod: boolean, name: string) {
  switch (type) {
    case ModalAlertType.failure:
      return newMethod
        ? sprintf(
            messages.pgettext(
              'api-access-methods-view',
              'The API could not be reached using the %(name)s method.',
            ),
            { name },
          )
        : sprintf(
            // TRANSLATORS: %(save)s - Will be replaced with the translation for the word "Save".
            messages.pgettext(
              'api-access-methods-view',
              'Clicking “%(save)s” changes the in use method.',
            ),
            { save: messages.gettext('Save') },
          );
    default:
      return undefined;
  }
}

function getTestingDialogButtons(type: ModalAlertType, save: () => void, cancel: () => void) {
  const saveButton = (
    <Button key="confirm" onClick={save}>
      <Button.Text>{messages.gettext('Save')}</Button.Text>
    </Button>
  );
  const cancelButton = (
    <Button key="cancel" onClick={cancel}>
      <Button.Text>{messages.gettext('Cancel')}</Button.Text>
    </Button>
  );
  const disabledCancelButton = (
    <Button key="cancel" onClick={cancel} disabled>
      <Button.Text>{messages.gettext('Cancel')}</Button.Text>
    </Button>
  );

  switch (type) {
    case ModalAlertType.success:
      return [disabledCancelButton];
    case ModalAlertType.failure:
      return [cancelButton, saveButton];
    case ModalAlertType.loading:
    default:
      return [cancelButton];
  }
}
