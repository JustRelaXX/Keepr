<script lang="ts">
  let {
    kind = 'plant',
    mood = 'happy',
    compact = false,
  }: { kind?: string; mood?: string; compact?: boolean } = $props();
  const happy = $derived(mood === 'happy' || mood === 'calm');
</script>

<div class="pet" class:compact class:happy aria-hidden="true">
  <svg viewBox="0 0 260 230" fill="none">
    <ellipse
      cx="131"
      cy="210"
      rx="62"
      ry="9"
      fill="currentColor"
      opacity=".08"
    />
    <g class="pet-body">
      {#if kind === 'plant'}
        <path
          d="M132 144V83"
          stroke="#5e8050"
          stroke-width="8"
          stroke-linecap="round"
        />
        <g class="pet-leaf"
          ><path
            d="M130 116C71 124 52 85 65 57c43-6 74 19 65 59Z"
            fill="#85af67"
          /><path
            d="M83 76q31 15 47 40"
            stroke="#629150"
            stroke-width="3"
            stroke-linecap="round"
          /></g
        >
        <g class="pet-leaf right-leaf"
          ><path
            d="M132 96C128 48 163 25 204 45c0 41-33 65-72 51Z"
            fill="#a7c97c"
          /><path
            d="M180 53q-26 12-47 42"
            stroke="#7fa45c"
            stroke-width="3"
            stroke-linecap="round"
          /></g
        >
        {#if mood === 'happy'}<g class="pet-flower" transform="translate(91 46)"
            ><path
              d="M0-12c8-9 17-1 13 7 12-2 15 11 6 15 4 10-8 15-14 8-6 9-17 2-13-7-11 0-14-13-4-17-1-11 10-15 12-6Z"
              fill="#f2ce7f"
            /><circle r="6" fill="#c58b52" /></g
          >{/if}
        <path
          d="M78 137h109l-12 59q-2 12-17 12h-52q-14 0-17-12Z"
          fill="#dfad92"
        />
        <path d="M81 153h102" stroke="#c18c73" stroke-width="3" opacity=".6" />
        <rect x="72" y="130" width="121" height="23" rx="9" fill="#edc4a9" />
        <path
          d="M110 169v6m42-6v6"
          stroke="#664e40"
          stroke-width="5"
          stroke-linecap="round"
        />
        <path
          d={happy ? 'M122 182q9 9 18 0' : 'M125 186h12'}
          stroke="#664e40"
          stroke-width="3"
          stroke-linecap="round"
        />
        <ellipse cx="102" cy="183" rx="7" ry="4" fill="#d99182" /><ellipse
          cx="161"
          cy="183"
          rx="7"
          ry="4"
          fill="#d99182"
        />
      {:else if kind === 'cat'}
        <path
          d="M178 180q46 8 39-31"
          stroke="#c9a380"
          stroke-width="18"
          stroke-linecap="round"
        />
        <ellipse cx="130" cy="168" rx="55" ry="40" fill="#d8b793" />
        <path
          d="m79 95 0-51 42 32m23 0 40-32 0 57"
          fill="#d8b793"
          stroke="#d8b793"
          stroke-width="8"
          stroke-linejoin="round"
        />
        <path d="m89 79 0-19 19 19m48 0 18-19 0 21" fill="#e9b4a3" />
        <ellipse cx="131" cy="112" rx="61" ry="49" fill="#dfbf99" />
        <path
          d="M104 106v7m52-7v7"
          stroke="#665240"
          stroke-width="5"
          stroke-linecap="round"
        />
        <path
          d="m126 122 5 4 5-4"
          fill="#b58079"
          stroke="#b58079"
          stroke-width="3"
          stroke-linejoin="round"
        />
        <path
          d={happy ? 'M118 130q7 8 13 0 6 8 13 0' : 'M122 134h18'}
          stroke="#665240"
          stroke-width="2.5"
          stroke-linecap="round"
        />
        <path
          d="m80 119-18-3m19 13-18 4m118-14 18-3m-19 13 18 4"
          stroke="#9a7c60"
          stroke-width="2"
          stroke-linecap="round"
        />
        <ellipse cx="108" cy="201" rx="19" ry="9" fill="#e7c9a7" /><ellipse
          cx="154"
          cy="201"
          rx="19"
          ry="9"
          fill="#e7c9a7"
        />
      {:else}
        <ellipse cx="130" cy="165" rx="60" ry="43" fill="#98b974" />
        <ellipse cx="130" cy="170" rx="39" ry="31" fill="#d3dfac" />
        <ellipse cx="131" cy="114" rx="70" ry="42" fill="#a6c782" />
        <circle cx="95" cy="83" r="25" fill="#a6c782" /><circle
          cx="165"
          cy="83"
          r="25"
          fill="#a6c782"
        />
        <circle cx="97" cy="84" r="13" fill="#eff1cc" /><circle
          cx="163"
          cy="84"
          r="13"
          fill="#eff1cc"
        />
        <path
          d="M99 81v7m62-7v7"
          stroke="#45633c"
          stroke-width="6"
          stroke-linecap="round"
        />
        <path
          d={happy ? 'M113 122q17 15 34 0' : 'M120 130h22'}
          stroke="#45633c"
          stroke-width="3"
          stroke-linecap="round"
        />
        <ellipse cx="90" cy="123" rx="9" ry="5" fill="#dbb391" /><ellipse
          cx="170"
          cy="123"
          rx="9"
          ry="5"
          fill="#dbb391"
        />
        <ellipse cx="85" cy="202" rx="25" ry="9" fill="#88ac65" /><ellipse
          cx="176"
          cy="202"
          rx="25"
          ry="9"
          fill="#88ac65"
        />
      {/if}
    </g>
    {#if mood === 'happy'}<g
        class="pet-sparkles"
        stroke="#bcaa5e"
        stroke-width="2.5"
        stroke-linecap="round"
        ><path d="M43 100v10m-5-5h10m162 14v10m-5-5h10" /><path
          d="M213 75v8m-4-4h8"
        /></g
      >{/if}
  </svg>
</div>
