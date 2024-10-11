import { useCallback, useEffect, useState } from 'react';
import { useParams } from 'react-router';
import { sprintf } from 'sprintf-js';

import {
  CustomProxy,
  NamedCustomProxy,
  NewAccessMethodSetting,
} from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { useScheduler } from '../../shared/scheduler';
import { useAppContext } from '../context';
import { useApiAccessMethodTest } from '../lib/api-access-methods';
import { useHistory } from '../lib/history';
import { useEffectEvent } from '../lib/utility-hooks';
import { useSelector } from '../redux/store';
import { SettingsForm } from './cell/SettingsForm';
import { BackAction } from './KeyboardNavigation';
import { Layout, SettingsContainer } from './Layout';
import { ModalAlert, ModalAlertType } from './Modal';
import { NavigationBar, NavigationContainer, NavigationItems, TitleBarItem } from './NavigationBar';
import { NamedProxyForm } from './ProxyForm';
import SettingsHeader, { HeaderSubTitle, HeaderTitle } from './SettingsHeader';
import { StyledContent, StyledNavigationScrollbars, StyledSettingsContent } from './SettingsStyles';
import { SmallButton } from './SmallButton';

export function EditApiAccessMethod() {
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

  const save = useCallback(() => {
    if (updatedMethod !== undefined) {
      resetTestResult();
      if (id === undefined) {
        void addApiAccessMethod(updatedMethod);
      } else {
        void updateApiAccessMethod({ ...updatedMethod, id });
      }
      pop();
    }
  }, [updatedMethod, resetTestResult, id, pop, addApiAccessMethod, updateApiAccessMethod]);

  const onSave = useCallback(
    async (newMethod: NamedCustomProxy) => {
      const enabled = id === undefined ? true : (method?.enabled ?? true);
      setUpdatedMethod({ ...newMethod, enabled });
      if (
        updatedMethod !== undefined &&
        (await testApiAccessMethod(updatedMethod as CustomProxy))
      ) {
        // Hide the save dialog after 1.5 seconds.
        saveScheduler.schedule(save, 1500);
      }
    },
    [id, method?.enabled, updatedMethod, testApiAccessMethod, saveScheduler, save],
  );

  const title = getTitle(id === undefined);
  const subtitle = getSubtitle(id === undefined);

  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <NavigationBar>
              <NavigationItems>
                <TitleBarItem>{title}</TitleBarItem>
              </NavigationItems>
            </NavigationBar>

            <StyledNavigationScrollbars fillContainer>
              <StyledContent>
                <SettingsHeader>
                  <HeaderTitle>{title}</HeaderTitle>
                  <HeaderSubTitle>{subtitle}</HeaderSubTitle>
                </SettingsHeader>

                <StyledSettingsContent>
                  {id !== undefined && method === undefined ? (
                    <span>Failed to open method</span>
                  ) : (
                    <NamedProxyForm proxy={method} onSave={onSave} onCancel={pop} />
                  )}
                </StyledSettingsContent>

                <TestingDialog
                  name={updatedMethod?.name ?? ''}
                  newMethod={id === undefined}
                  testing={testing}
                  testResult={testResult}
                  cancel={resetTestResult}
                  save={save}
                />
              </StyledContent>
            </StyledNavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
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
  const calculatedType = props.testing
    ? ModalAlertType.loading
    : props.testResult
      ? ModalAlertType.success
      : ModalAlertType.failure;

  const [type, setType] = useState(calculatedType);
  const isOpen = props.testing || props.testResult !== undefined;

  const typeChangeEvent = useEffectEvent((type: ModalAlertType) => {
    if (isOpen) {
      setType(type);
    }
  });

  useEffect(() => typeChangeEvent(calculatedType), [calculatedType]);

  return (
    <ModalAlert
      isOpen={isOpen}
      type={type}
      gridButtons={getTestingDialogButtons(type, props.save, props.cancel)}
      close={props.cancel}
      title={getTestingDialogTitle(type, props.newMethod)}
      message={getTestingDialogSubTitle(type, props.newMethod, props.name)}
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
    <SmallButton key="confirm" onClick={save}>
      {messages.gettext('Save')}
    </SmallButton>
  );
  const cancelButton = (
    <SmallButton key="cancel" onClick={cancel}>
      {messages.gettext('Cancel')}
    </SmallButton>
  );
  const disabledCancelButton = (
    <SmallButton key="cancel" onClick={cancel} disabled>
      {messages.gettext('Cancel')}
    </SmallButton>
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
