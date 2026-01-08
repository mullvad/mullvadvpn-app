import { describe, expect, it, vi } from 'vitest';

import KeyframeAnimation from '../../src/main/keyframe-animation';

describe('lib/keyframe-animation', function () {
  vi.setConfig({ testTimeout: 1000 });

  const newAnimation = () => {
    const animation = new KeyframeAnimation();
    animation.speed = 1;
    return animation;
  };

  it('should play sequence', () => {
    return new Promise((resolve) => {
      const seq: number[] = [];
      const animation = newAnimation();
      animation.onFrame = (frame) => {
        seq.push(frame);
      };
      animation.onFinish = () => {
        expect(seq).to.be.deep.equal([0, 1, 2, 3, 4]);
        expect(animation.currentFrame).to.be.equal(4);
        resolve(null);
      };

      animation.play({ end: 4 });
    });
  });

  it('should play one frame', () => {
    return new Promise((resolve) => {
      const seq: number[] = [];
      const animation = newAnimation();
      animation.onFrame = (frame) => {
        seq.push(frame);
      };
      animation.onFinish = () => {
        expect(seq).to.be.deep.equal([3]);
        expect(animation.currentFrame).to.be.equal(3);
        resolve(null);
      };

      animation.play({ start: 3, end: 3 });
    });
  });

  it('should play sequence with custom frames', () => {
    return new Promise((resolve) => {
      const seq: number[] = [];
      const animation = newAnimation();
      animation.onFrame = (frame) => {
        seq.push(frame);
      };
      animation.onFinish = () => {
        expect(seq).to.be.deep.equal([2, 3, 4]);
        expect(animation.currentFrame).to.be.equal(4);
        resolve(null);
      };

      animation.play({ start: 2, end: 4 });
    });
  });

  it('should play sequence with custom frames in reverse', () => {
    return new Promise((resolve) => {
      const seq: number[] = [];
      const animation = newAnimation();
      animation.onFrame = (frame) => {
        seq.push(frame);
      };
      animation.onFinish = () => {
        expect(seq).to.be.deep.equal([4, 3, 2]);
        expect(animation.currentFrame).to.be.equal(2);
        resolve(null);
      };

      animation.play({ start: 4, end: 2 });
    });
  });

  it('should begin from current state starting below range', () => {
    return new Promise((resolve) => {
      const seq: number[] = [];
      const animation = newAnimation();
      animation.onFrame = (frame) => {
        seq.push(frame);
      };
      animation.onFinish = () => {
        expect(seq).to.be.deep.equal([0, 1, 2, 3, 4]);
        expect(animation.currentFrame).to.be.equal(4);
        resolve(null);
      };

      animation.currentFrame = 0;
      animation.play({ end: 4 });
    });
  });

  it('should begin from current state starting above range', () => {
    return new Promise((resolve) => {
      const seq: number[] = [];
      const animation = newAnimation();
      animation.onFrame = (frame) => {
        seq.push(frame);
      };
      animation.onFinish = () => {
        expect(seq).to.be.deep.equal([4, 3, 2]);
        expect(animation.currentFrame).to.be.equal(2);
        resolve(null);
      };

      animation.currentFrame = 4;
      animation.play({ end: 2 });
    });
  });

  it('should begin from current state starting above range reverse', () => {
    return new Promise((resolve) => {
      const seq: number[] = [];
      const animation = newAnimation();
      animation.onFrame = (frame) => {
        seq.push(frame);
      };
      animation.onFinish = () => {
        expect(seq).to.be.deep.equal([4, 3, 2, 1]);
        expect(animation.currentFrame).to.be.equal(1);
        resolve(null);
      };

      animation.currentFrame = 4;
      animation.play({ end: 1 });
    });
  });

  it('should play sequence in reverse', () => {
    return new Promise((resolve) => {
      const seq: number[] = [];
      const animation = newAnimation();
      animation.onFrame = (frame) => {
        seq.push(frame);
      };
      animation.onFinish = () => {
        expect(seq).to.be.deep.equal([4, 3, 2, 1, 0]);
        expect(animation.currentFrame).to.be.equal(0);
        resolve(null);
      };

      animation.play({ start: 4, end: 0 });
    });
  });
});
