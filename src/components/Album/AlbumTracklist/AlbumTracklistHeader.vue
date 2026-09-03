<template>
  <div
    class="grid grid-cols-[3rem_minmax(0,1fr)_4rem] items-center gap-4 px-4 pb-3 font-sans text-xs font-semibold tracking-wide text-zinc-500 uppercase @min-[48rem]:grid-cols-[3rem_minmax(0,1fr)_11rem_4rem]"
  >
    <span class="grid size-6 place-items-center">#</span>
    <span>Title</span>
    <span class="hidden @min-[48rem]:block">Rating</span>
    <span class="text-right">Time</span>
  </div>
</template>
