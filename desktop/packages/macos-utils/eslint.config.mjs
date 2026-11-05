import workspaceConfig from '../../eslint.config.mjs';

export default [...workspaceConfig, { ignores: ['lib/'] }];
