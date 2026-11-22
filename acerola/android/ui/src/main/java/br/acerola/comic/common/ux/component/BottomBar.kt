package br.acerola.comic.common.ux.component

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.navigation.NavHostController
import androidx.navigation.compose.currentBackStackEntryAsState
import br.acerola.comic.common.navigation.Destination
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.modifier.float
import br.acerola.comic.common.ux.theme.AcerolaExtendedTheme
import br.acerola.comic.common.ux.theme.AcerolaShapes
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens
import dev.chrisbanes.haze.HazeState
import dev.chrisbanes.haze.HazeStyle
import dev.chrisbanes.haze.HazeTint
import dev.chrisbanes.haze.hazeEffect

private val navDestinations =
    listOf(
        Destination.HOME,
        Destination.HISTORY,
        Destination.SYNC,
        Destination.CONFIG,
    )

/**
 * Brand pill navigation (`AcerolaPillNav`, `.ac-pill-nav`): a floating `bg-card/95` pill with
 * `shadow-float` and a backdrop blur; each item is a 40dp round icon, the active one becomes a
 * `bg-primary` pill with its name. Takes 64dp (52dp pill + 12dp gap) over the navigation bar.
 */
@Composable
fun Acerola.Component.BottomBar(
    navController: NavHostController,
    hazeState: HazeState,
) {
    val navBackStackEntry by navController.currentBackStackEntryAsState()
    val currentRoute = navBackStackEntry?.destination?.route
    val brand = AcerolaExtendedTheme.colors

    Box(
        modifier =
            Modifier
                .fillMaxWidth()
                .windowInsetsPadding(WindowInsets.navigationBars)
                .padding(bottom = SpacingTokens.Medium),
        contentAlignment = Alignment.Center,
    ) {
        Row(
            modifier =
                Modifier
                    .float(AcerolaShapes.Full)
                    .clip(AcerolaShapes.Full)
                    .hazeEffect(
                        state = hazeState,
                        style =
                            HazeStyle(
                                backgroundColor = brand.card,
                                tints = listOf(HazeTint(color = brand.card.copy(alpha = 0.95f))),
                                blurRadius = 24.dp,
                            ),
                    ).padding(6.dp),
            horizontalArrangement = Arrangement.spacedBy(SpacingTokens.ExtraSmall),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            navDestinations.forEach { destination ->
                val routeString = stringResource(id = destination.route)
                val selected = currentRoute == routeString
                val container by animateColorAsState(if (selected) MaterialTheme.colorScheme.primary else Color.Transparent, label = "pillNavBg")
                val content by animateColorAsState(
                    if (selected) MaterialTheme.colorScheme.onPrimary else brand.mutedForeground,
                    label = "pillNavFg",
                )

                Row(
                    modifier =
                        Modifier
                            .height(40.dp)
                            .widthIn(min = 40.dp)
                            .clip(AcerolaShapes.Full)
                            .background(container)
                            .semantics { this.selected = selected }
                            .clickable {
                                if (!selected) {
                                    navController.navigate(routeString) {
                                        popUpTo(navController.graph.startDestinationId) { saveState = true }
                                        launchSingleTop = true
                                        restoreState = true
                                    }
                                }
                            }.padding(horizontal = SpacingTokens.Medium)
                            .animateContentSize(),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small, Alignment.CenterHorizontally),
                ) {
                    Icon(
                        imageVector = destination.icon,
                        contentDescription = stringResource(destination.contentDescriptionRes),
                        tint = content,
                        modifier = Modifier.size(SizeTokens.IconSmall),
                    )
                    if (selected) {
                        Text(
                            text = stringResource(id = destination.label),
                            style = MaterialTheme.typography.labelLarge.copy(fontWeight = FontWeight.Bold),
                            color = content,
                            maxLines = 1,
                        )
                    }
                }
            }
        }
    }
}

@Composable
fun Acerola.Component.SideBar(navController: NavHostController) {
    val navBackStackEntry by navController.currentBackStackEntryAsState()
    val currentRoute = navBackStackEntry?.destination?.route

    // Insets: start (status bar top) + bottom (navigation bar), but NOT end
    // so the sidebar goes edge-to-edge on its own side like Spotify
    val sideBarInsets =
        WindowInsets.safeDrawing.only(
            WindowInsetsSides.Start + WindowInsetsSides.Vertical,
        )

    // Outer Box: fills the full height and applies the background color (including behind the status bar and nav bar)
    Box(
        modifier =
            Modifier
                .fillMaxHeight()
                .background(AcerolaExtendedTheme.colors.mantle),
    ) {
        Column(
            modifier =
                Modifier
                    .fillMaxHeight()
                    .windowInsetsPadding(sideBarInsets)
                    .width(200.dp)
                    .padding(vertical = 12.dp, horizontal = 8.dp),
            verticalArrangement = Arrangement.Center,
        ) {
            navDestinations.forEach { destination ->
                val routeString = stringResource(id = destination.route)
                val isSelected = currentRoute == routeString
                // Same as the desktop sidebar: round items, the active one a `bg-primary` pill.
                val itemBackground = if (isSelected) MaterialTheme.colorScheme.primary else Color.Transparent
                val contentColor =
                    if (isSelected) MaterialTheme.colorScheme.onPrimary else AcerolaExtendedTheme.colors.mutedForeground

                Box(
                    modifier =
                        Modifier
                            .fillMaxWidth()
                            .clip(AcerolaShapes.Full)
                            .background(itemBackground)
                            .clickable {
                                if (currentRoute != routeString) {
                                    navController.navigate(routeString) {
                                        popUpTo(navController.graph.startDestinationId) { saveState = true }
                                        launchSingleTop = true
                                        restoreState = true
                                    }
                                }
                            }.padding(horizontal = 16.dp, vertical = 12.dp),
                ) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Icon(
                            imageVector = destination.icon,
                            contentDescription = stringResource(destination.contentDescriptionRes),
                            tint = contentColor,
                            modifier = Modifier.size(24.dp),
                        )
                        Spacer(modifier = Modifier.width(12.dp))
                        Text(
                            text = stringResource(id = destination.label),
                            style = MaterialTheme.typography.labelLarge.copy(fontWeight = FontWeight.Bold),
                            color = contentColor,
                        )
                    }
                }

                Spacer(modifier = Modifier.size(4.dp))
            }
        }
    }
}
