// place files you want to import through the `$lib` alias in this folder.
import './chunk';
import './crypto';
import { uploadFile } from './upload';
import { createCapabilityUrl } from './cap_url';

export default { uploadFile, createCapabilityUrl };
