<script lang="ts">
  import { colors, icons } from '../options';
  import { t } from '../i18n.svelte';
  import Icon from './Icon.svelte';
  let {
    icon = $bindable(),
    color = $bindable(),
  }: { icon: string; color: string } = $props();
</script>

<fieldset class="appearance">
  <legend>{t('appearance')}</legend>
  <div class="color-options" role="group" aria-label={t('color')}>
    {#each colors as c}<button
        type="button"
        data-color={c}
        class:chosen={color === c}
        aria-pressed={color === c}
        aria-label={t(`color_${c}`)}
        onclick={() => (color = c)}
        >{#if color === c}<Icon name="check" size={17} />{/if}</button
      >{/each}
  </div>
  <div class="icon-options" role="group" aria-label={t('icon')}>
    {#each icons as name, i}<button
        type="button"
        class:chosen={icon === name}
        aria-pressed={icon === name}
        aria-label="{t('choose_icon')} {i + 1}"
        onclick={() => (icon = name)}><Icon {name} size={21} /></button
      >{/each}
  </div>
</fieldset>
