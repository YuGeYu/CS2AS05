/**
 * 统一的水墨江南鸣谢场景入口。
 *
 * 开屏和版本彩蛋必须共享同一套 Three.js 世界、牌面构建和相机规则；
 * 这里保留旧模块的实现位置，只把公共工厂收敛到一个稳定入口，避免
 * 两个调用方以后再次出现视觉或数据分叉。
 */
export { createIntroScene as createAcknowledgementScene, introCardSignature } from '@/features/intro/scene'
export type { IntroSceneState as AcknowledgementSceneState } from '@/features/intro/scene'
