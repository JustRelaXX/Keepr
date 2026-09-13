<script lang="ts">
  import Icon from './Icon.svelte';
  let { icon, size = 48 }: { icon: string; size?: number } = $props();
</script>

<span
  class="thing-art"
  style:width="{size}px"
  style:height="{size}px"
  aria-hidden="true"
>
  {#if ['brush', 'droplets', 'sponge', 'bed', 'cloud', 'bottle', 'milk', 'sprout'].includes(icon)}
    <svg viewBox="0 0 64 64" fill="none">
      {#if icon === 'brush'}
        <g transform="rotate(28 32 32)"
          ><rect
            x="28"
            y="24"
            width="9"
            height="34"
            rx="4.5"
            fill="#739bc4"
          /><rect
            x="26"
            y="6"
            width="14"
            height="23"
            rx="5"
            fill="#d4e7f5"
            stroke="#507ba4"
            stroke-width="2"
          /><path
            d="M28 11h9m-9 5h9m-9 5h9"
            stroke="#79a3c4"
            stroke-width="2"
            stroke-linecap="round"
          /></g
        >
      {:else if icon === 'droplets'}
        <rect
          x="16"
          y="16"
          width="33"
          height="39"
          rx="11"
          fill="#b3d5e9"
          stroke="#507c9a"
          stroke-width="2"
        /><rect
          x="18"
          y="10"
          width="29"
          height="10"
          rx="4"
          fill="#e5f1f5"
          stroke="#507c9a"
          stroke-width="2"
        /><path
          d="M32 25c-4 6-8 8-8 12a8 8 0 0 0 16 0c0-4-4-6-8-12Z"
          fill="#6b9bbb"
        /><path
          d="M24 49h16"
          stroke="#507c9a"
          stroke-width="2"
          stroke-linecap="round"
        />
      {:else if icon === 'sponge'}
        <path
          d="M10 27q0-8 8-8h30q8 0 8 8v15q0 8-8 8H18q-8 0-8-8Z"
          fill="#ecd17f"
          stroke="#b19b4f"
          stroke-width="2"
        /><path d="M11 38h44v7q0 6-8 6H18q-7 0-7-6Z" fill="#8aaa70" /><circle
          cx="21"
          cy="28"
          r="2.5"
          fill="#d5b658"
        /><circle cx="44" cy="30" r="3" fill="#d5b658" /><circle
          cx="32"
          cy="24"
          r="1.5"
          fill="#d5b658"
        />
      {:else if icon === 'bed' || icon === 'cloud'}
        <path
          d="M12 19q20-5 40 0l3 29q-23 5-46 0Z"
          fill="#d3c9ec"
          stroke="#9686b2"
          stroke-width="2"
        /><path d="M15 24q17-4 34 0l1 18q-18 4-36 0Z" fill="#eee9f8" /><path
          d="m13 20 3 4m34-4-3 4m-36 22 4-4m36 4-4-4"
          stroke="#9686b2"
          stroke-width="2"
        />
      {:else if icon === 'bottle' || icon === 'milk'}
        <rect
          x="20"
          y="19"
          width="25"
          height="37"
          rx="7"
          fill="#e7bda7"
          stroke="#ad836e"
          stroke-width="2"
        /><rect
          x="25"
          y="9"
          width="15"
          height="13"
          rx="3"
          fill="#d0a98c"
          stroke="#ad836e"
          stroke-width="2"
        /><rect
          x="23"
          y="31"
          width="19"
          height="16"
          rx="3"
          fill="#fff4df"
        /><path
          d="m29 37 3 4 5-6"
          stroke="#a37c62"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      {:else}
        <path
          d="M33 40V19"
          stroke="#648454"
          stroke-width="3"
          stroke-linecap="round"
        /><path
          d="M32 30C15 33 9 21 14 15c11-1 19 5 18 15Z"
          fill="#95b573"
        /><path
          d="M33 23C34 8 47 6 52 12c-1 10-9 15-19 11Z"
          fill="#699857"
        /><path
          d="M18 38h29l-4 18H22Z"
          fill="#d6a08b"
          stroke="#a87563"
          stroke-width="2"
        /><path
          d="M18 38h29"
          stroke="#a87563"
          stroke-width="4"
          stroke-linecap="round"
        />
      {/if}
    </svg>
  {:else}<Icon name={icon} size={size * 0.62} stroke={1.5} />{/if}
</span>
