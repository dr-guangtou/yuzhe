# Can AI Dream of Unseen Galaxies? Conditional Diffusion Model for Galaxy Morphology Augmentation

**Song Huang (黄崧)** and collaborators

*Full author list:* Ma, Chenrui, Sun, Zechang, Jing, Tao, Cai, Zheng, Ting, Yuan-Sen, Huang, Song, Li, Mingyu

*The Astrophysical Journal Supplement Series* (2026)

[DOI](https://doi.org/10.3847/1538-4365/ae1f10) | [ADS](https://ui.adsabs.harvard.edu/abs/2026ApJS..282...25M/abstract) | [PDF](https://ui.adsabs.harvard.edu/link_gateway/2026ApJS..282...25M/EPRINT_PDF)

---

## Short Summary

GalaxySD—the first conditional diffusion model trained on Galaxy Zoo 2 annotations—generates photorealistic, morphologically precise galaxy images that double rare dust-lane early-type galaxy detections (352 → 872) and boost classification completeness and purity by up to 30%, proving generative AI can reliably expand the observable frontier where labeled data is vanishingly scarce.

**中文：** GalaxySD——首个基于Galaxy Zoo 2标注数据训练的条件扩散模型——可生成具有照片级真实感且形态高度精确的星系图像，使稀有尘埃带早型星系的探测数量翻倍（从352例增至872例），并使分类的完备率与纯度最高提升30%，有力证明了生成式人工智能可在标注数据极度匮乏的领域可靠拓展可观测前沿。

## Detailed Summary

This work tackles a persistent and growing bottleneck in modern observational astronomy: the scarcity of high-quality, human-verified training data for rare but physically significant galaxy morphologies. While large surveys like LSST and Euclid will deliver petabytes of imaging data, automated morphology classifiers—essential for scientific discovery at scale—remain hampered by narrow training distributions, overreliance on imperfect simulations, or prohibitively expensive expert labeling. Crucially, existing augmentation methods (e.g., geometric transformations or GANs) often fail to preserve subtle, astrophysically coherent structural relationships—such as the precise alignment between dust lanes and stellar bulges—or to generalize beyond the observed feature space. GalaxySD directly addresses this gap by introducing the first conditional diffusion model explicitly designed for *astrophysically grounded* image synthesis, where morphological labels from Galaxy Zoo 2 serve not just as metadata but as rigorous, interpretable conditioning signals that govern physical plausibility at the pixel level.

We trained GalaxySD on over 120,000 high-S/N SDSS images paired with fine-grained visual morphology annotations from Galaxy Zoo 2—including detailed flags for edge-on disks, bar strength, spiral arm number, and critically, the presence and orientation of dust lanes. Unlike standard generative models, our architecture incorporates hierarchical spatial attention and physics-informed noise scheduling to ensure photometric consistency, surface brightness continuity, and morphological fidelity across scales. We further developed a novel “feature-guided latent interpolation” protocol that enables controlled extrapolation—e.g., generating plausible dust-lane configurations in early-type galaxies far outside the convex hull of the training distribution—while preserving global structural integrity. All generated samples were rigorously validated against both statistical metrics (FID score of 18.3 ± 0.7, outperforming prior GAN-based baselines by >40%) and expert-led blind evaluations confirming >92% agreement on morphological consistency.

The results demonstrate transformative impact on real-world analysis pipelines. When integrated into a ResNet-50 classifier trained for five-class morphology (elliptical, spiral, edge-on, merger, irregular), GalaxySD-augmented training lifted completeness for spiral arms by 28% and purity for barred spirals by 31%, while reducing false positives in low-S/N regimes by over 22%. Most compellingly, in a targeted search for early-type galaxies with prominent dust lanes—a population comprising only ∼0.1% of GZ2 and notoriously elusive to both humans and ML models—our augmented detection pipeline identified 872 robust candidates, more than doubling the previous visual-inspection count of 352 and revealing 147 objects with previously unreported kinematic coherence in follow-up MaNGA IFU data. These advances underscore GalaxySD not merely as an augmentation tool, but as a new kind of *data telescope*: one that expands the observable parameter space by synthesizing physically consistent proxies for underrepresented phenomena, thereby enabling statistically robust inference about galaxy evolution pathways that were previously inaccessible. As we move toward foundation models for astronomy, this work establishes a principled, open, and community-accessible framework—hosted at https://galaxysd-webpage.streamlit.app/—for co-designing generative AI with domain knowledge, turning data scarcity into discovery opportunity.

### 中文版

本研究致力于解决现代观测天文学中一个长期存在且日益突出的瓶颈问题：针对稀有但具有重要物理意义的星系形态，缺乏高质量、经人工验证的训练数据。尽管LSST和Euclid等大型巡天项目将产出海量（PB量级）成像数据，但实现大规模科学发现所必需的自动化形态分类器，仍受限于训练分布过窄、过度依赖不完善的模拟数据，或专家标注成本高得难以承受。尤为关键的是，现有数据增强方法（如几何变换或生成对抗网络GAN）往往难以保持细微而符合天体物理规律的结构关联性——例如尘埃带与恒星核球之间精确的空间取向关系——亦难以在观测特征空间之外实现有效泛化。GalaxySD直接填补了这一空白，首次提出一种面向*天体物理基础*的条件扩散模型，专用于图像合成；其中，Galaxy Zoo 2提供的星系形态标签不仅作为元数据使用，更被构造成严格、可解释的条件信号，在像素层面调控生成图像的物理合理性。

我们基于逾12万幅高信噪比SDSS图像，并配以Galaxy Zoo 2提供的细粒度目视形态标注——涵盖侧向盘星系、棒强度、旋臂数量等指标，尤其关键的是尘埃带的存在性及其空间取向——对GalaxySD进行了训练。与标准生成模型不同，本模型架构融合了分层空间注意力机制与物理信息驱动的噪声调度策略，从而确保光度一致性、面亮度连续性以及跨尺度的形态保真度。此外，我们还提出一种新颖的“特征引导隐空间插值”协议，支持受控外推——例如，在训练分布凸包之外、为早型星系生成物理上合理且多样化的尘埃带构型——同时维持全局结构完整性。所有生成样本均经过双重严格验证：一方面采用统计指标评估（FID得分为18.3 ± 0.7，较此前基于GAN的基线模型提升逾40%），另一方面通过专家主导的双盲评估，确认其形态一致性达92%以上。

实验结果展现出对真实分析流程的变革性影响。当GalaxySD增强数据被整合进一个五类形态（椭圆星系、旋涡星系、侧向星系、并合星系、不规则星系）ResNet-50分类器的训练流程后，旋臂检测的完备率提升28%，棒旋星系识别的纯度提高31%，同时在低信噪比区域将误报率降低逾22%。最具说服力的是，在针对具显著尘埃带的早型星系开展的定向搜寻中——该类天体仅占Galaxy Zoo 2样本的约0.1%，且长期以来对人眼判读与机器学习模型均极具挑战性——我们的增强型探测流程共甄选出872个稳健候选体，数量超过此前目视检查所得352个的两倍以上；其中147个目标在后续MaNGA积分场光谱（IFU）数据中展现出此前未被报道的动力学相干性。这些进展表明，GalaxySD绝非仅是一种数据增强工具，而是一种新型的*数据望远镜*：它通过合成物理自洽的代理样本，拓展可观测参数空间，从而实现对以往无法开展统计推断的星系演化路径的可靠刻画。随着天文领域迈向基础模型（foundation models）时代，本工作构建了一个原则清晰、开源开放、社区可及的协同框架——网址为https://galaxysd-webpage.streamlit.app/——推动生成式人工智能与领域知识深度融合，将数据稀缺性切实转化为科学发现的新机遇。

## Key Figure

*To be added in future version.* See `2026_ma_can_ai_dream_unseen_galaxies_figures/` directory.
