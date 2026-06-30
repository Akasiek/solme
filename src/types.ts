export interface ServerInfo {
  serverType: string;
  serverVersion?: string;
  apiVersion: string;
  username: string;
}

export interface SavedServerProfile {
  serverType: string;
  url: string;
  username: string;
}

export interface LibrarySyncStatus {
  phase: "idle" | "metadata" | "activating" | "artwork" | "completed" | "failed";
  processedArtists: number;
  processedAlbums: number;
  processedSongs: number;
  processedArtwork: number;
  totalArtwork: number;
  lastSuccessAt?: number;
  lastError?: string;
}

export interface LibrarySummary {
  artistCount: number;
  albumCount: number;
  songCount: number;
  lastSuccessAt?: number;
}

export interface CachedAlbum {
  remoteId: string;
  name: string;
  artistName: string;
  artistId?: string;
  year?: number;
  songCount: number;
  artworkPath?: string;
}

export interface CachedSong {
  remoteId: string;
  albumId: string;
  title: string;
  artistName: string;
  artistId?: string;
  albumName: string;
  artworkPath?: string;
  trackNumber?: number;
  discNumber?: number;
  durationSeconds: number;
}

export interface PlayerStatus {
  state: "stopped" | "playing" | "paused";
  currentSong?: CachedSong;
  positionSeconds: number;
  durationSeconds: number;
  queuePosition?: number;
  queueLength: number;
  volume: number;
}
