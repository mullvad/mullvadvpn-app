import { nativeImage } from 'electron';
import path from 'path';

export class TrayIcon {
  constructor(public fileName?: string) {}

  public get basePath() {
    const basePath = path.resolve(import.meta.dirname, 'assets/images/menubar-icons');

    return basePath;
  }

  public get extension() {
    const extension = process.platform === 'win32' ? 'ico' : 'png';

    return extension;
  }

  public get filePath() {
    if (this.fileName) {
      const filePath = path.join(
        this.basePath,
        process.platform,
        `${this.fileName}.${this.extension}`,
      );

      return filePath;
    }

    return null;
  }

  public toNativeImage() {
    if (this.filePath) {
      return nativeImage.createFromPath(this.filePath);
    }

    return nativeImage.createEmpty();
  }
}
